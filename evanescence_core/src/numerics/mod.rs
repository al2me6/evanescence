//! Functions and traits relating to numerical evaluation.

use std::ops::RangeInclusive;

use itertools::Itertools;

use crate::geometry::{ComponentForm, GridValues, Linspace, Plane, Point, PointValue, Vec3};

/// Verify that two iterables containing float values are approximately equal.
#[cfg(test)]
macro_rules! assert_iterable_relative_eq {
    ($lhs:expr, $rhs: expr $(, $opt:ident = $val:expr)* $(,)?) => {{
        assert_eq!($lhs.len(), $rhs.len());
        assert!(
            $lhs.iter()
                .zip($rhs.iter())
                .all(|(l, r)| approx::relative_eq!(l, r $(, $opt = $val)*)),
            "assertion failed: `(left ≈ right)`\n\
                left: `{:?}`\n\
                right: `{:?}`",
            $lhs,
            $rhs
        );
    }};
}

pub mod orthogonal_polynomials;

/// Compute the [double factorial](https://en.wikipedia.org/wiki/Double_factorial).
pub trait DoubleFactorial {
    /// `x!!`
    #[must_use]
    fn double_factorial(self) -> Self;
}

macro_rules! impl_double_factorial {
    ($($T:ty),+) => {
        $(impl DoubleFactorial for $T {
            #[inline]
            fn double_factorial(self) -> Self {
                if self <= 1 {
                    return 1;
                }
                let mut acc = self;
                let delta = 2;
                let mut mul = acc - delta;
                while mul >= delta {
                    acc *= mul;
                    mul -= delta;
                }
                acc
            }
        })+
    }
}
impl_double_factorial!(u8, u16, u32, u64, usize);

/// Map `val`, which has a value within `source_range`, to `target_range`.
pub fn normalize(
    source_range: RangeInclusive<f32>,
    target_range: RangeInclusive<f32>,
    val: f32,
) -> f32 {
    (val - source_range.start()) / (source_range.end() - source_range.start())
        * (target_range.end() - target_range.start())
        + target_range.start()
}

/// Apply [`normalize`] to an entire collection of values.
pub fn normalize_collection<'a>(
    source_range: RangeInclusive<f32>,
    target_range: RangeInclusive<f32>,
    vals: impl IntoIterator<Item = &'a mut f32>,
) {
    vals.into_iter()
        .for_each(|v| *v = normalize(source_range.clone(), target_range.clone(), *v));
}

/// A potato trapezoidal integrator computing the integral of `y dx` given existing samples.
///
/// `xs` and `ys` must have the same length, and `xs` should be monotonically increasing.
pub fn integrate_trapezoidal(xs: &[f32], ys: &[f32]) -> f32 {
    itertools::zip_eq(xs.array_windows(), ys.array_windows())
        .map(|([x1, x2], [y1, y2])| (y1 + y2) * 0.5 * (x2 - x1))
        .sum()
}

/// Perform a single step of RK4 integration at step size `h`, accumulating into `t` and `y`.
///
/// # Panics
/// Panics if the step size `h` is not positive.
pub fn integrate_rk4_step(dfdt: impl Fn(f32) -> f32, t: &mut f32, y: &mut f32, h: f32) {
    const FRAC_1_6: f32 = 0.166_666_67;

    assert!(h > 0_f32);

    let k_1 = dfdt(*t);
    let k_2 = dfdt(*t + 0.5 * h);
    let k_4 = dfdt(*t + h);
    *y += FRAC_1_6 * h * (k_1 + 4_f32 * k_2 + k_4);
    *t += h;
}

/// Integrate `df(t)/dt` on `[t_0, t_f]` with step size `h` using the RK4 method.
///
/// # Panics
/// Panics if the step size `h` is not positive, or if the interval of integration is backwards.
pub fn integrate_rk4(dfdt: impl Fn(f32) -> f32, t_0: f32, t_f: f32, mut h: f32) -> f32 {
    assert!(t_0 < t_f);
    assert!(h > 0_f32);

    let mut t = t_0;
    let mut y = 0_f32;
    while t < t_f {
        h = h.min(t_f - t);
        integrate_rk4_step(&dfdt, &mut t, &mut y, h);
    }
    y
}

/// Evaluate a multiple integral as an iterated integral using [`integrate_rk4`]. The integrals are
/// evaluated in the order that the bounds for the variables are listed.
///
/// Syntax:
/// ```ignore
/// integrate_rk4_multiple!(
///     <expression in several variables>,
///     step: <step size>,
///     <first variable>: (<lower bound>, <upper bound>),
///     // etc.
/// )
/// ```
#[macro_export]
macro_rules! integrate_rk4_multiple {
    (
        $f:expr,
        step : $h:expr,
        $var:ident : ($t_0:expr , $t_f:expr)
        $(,)?
    ) => {
        $crate::numerics::integrate_rk4(|$var| $f, $t_0, $t_f, $h)
    };
    (
        $f:expr,
        step : $h:expr,
        $var:ident : ($t_0:expr , $t_f:expr) ,
        $($var_tail:ident : $range_tail:tt),+
        $(,)?
    ) => {
        $crate::numerics::integrate_rk4(
            |$var| integrate_rk4_multiple!(
                $f,
                step : $h,
                $($var_tail : $range_tail),+
            ),
            $t_0,
            $t_f,
            $h,
        )
    }
}

/// Trait for mathematical functions that can be evaluated at a point in `R^3`.
///
/// Utilities are provided for sampling the function on a line or plane.
pub trait Evaluate {
    type Parameters: Clone;
    type Output: Copy;

    /// Evaluate `Self` at a certain point, returning the value only.
    fn evaluate(params: &Self::Parameters, point: &Point) -> Self::Output;

    /// Evaluate `Self` at a certain point, returning the point *and* the value in the form of a
    /// [`PointValue`], or a `(Point, Self::Output)`.
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn evaluate_at(params: &Self::Parameters, point: &Point) -> PointValue<Self::Output> {
        PointValue(*point, Self::evaluate(params, point))
    }

    /// Evaluate `Self` on a line segment running across `range` at a total of `num_points`
    /// different points, all evenly spaced (à la "`linspace`" operation).
    fn evaluate_on_line_segment(
        params: &Self::Parameters,
        range: RangeInclusive<Vec3>,
        num_points: usize,
    ) -> Vec<PointValue<Self::Output>> {
        range
            .linspace(num_points)
            .map(|pt| Self::evaluate_at(params, &pt.into()))
            .collect()
    }

    /// Evaluate `Self` on a [`Plane`], producing a [grid](crate::geometry::GridValues) of evenly
    /// spaced values. Specifically, the grid is a square centered at the origin with side
    /// length of 2 × `extent`, and `num_points` are sampled *in each dimension*.
    fn evaluate_on_plane(
        params: &Self::Parameters,
        plane: Plane,
        extent: f32,
        num_points: usize,
    ) -> GridValues<Self::Output> {
        type ComponentGetter = fn(&Vec3) -> f32;
        // Functions to extract the correct component of the `Vec3`.
        let extract_component: (ComponentGetter, ComponentGetter) = match plane {
            Plane::XY => (Vec3::get_x, Vec3::get_y),
            Plane::YZ => (Vec3::get_y, Vec3::get_z),
            Plane::ZX => (Vec3::get_z, Vec3::get_x),
        };

        // The midpoints of the grid's "right" and "top" edges.
        let midpoints = {
            let basis = plane.basis_vectors();
            (basis.0 * extent, basis.1 * extent)
        };

        // Points linearly dependent on `e_0`, i.e., the center row.
        let points_in_row: Vec<_> = Vec3::symmetric_linspace(midpoints.0, num_points).collect();
        // Points linearly dependent on `e_1`, i.e., the center column.
        let points_in_col: Vec<_> = Vec3::symmetric_linspace(midpoints.1, num_points).collect();

        let mut vals = Vec::with_capacity(num_points);

        for &col_pt in &points_in_col {
            let mut row = Vec::with_capacity(num_points);
            for &row_pt in &points_in_row {
                row.push(Self::evaluate(params, &(row_pt + col_pt).into()));
            }
            vals.push(row);
        }

        GridValues::new(
            plane,
            points_in_row.iter().map(extract_component.0).collect(),
            points_in_col.iter().map(extract_component.1).collect(),
            vals,
        )
        .expect("rows and columns are equal in length by construction")
    }

    /// Evaluate `Self` on a cube of side length 2 × `extent`, centered at the origin, producing
    /// a list of evenly spaced points arranged as a flattened 3D array, with the first index
    /// being x, second index being y, and third index being z.
    ///
    /// That is, values are each of the form (x, y, z, val), sorted by increasing x, then y, and
    /// finally z.
    fn evaluate_in_region(
        params: &Self::Parameters,
        extent: f32,
        num_points: usize,
    ) -> ComponentForm<Self::Output> {
        Vec3::symmetric_linspace(Vec3::I * extent, num_points)
            .flat_map(|x_pt| {
                Vec3::symmetric_linspace(Vec3::J * extent, num_points).flat_map(move |y_pt| {
                    Vec3::symmetric_linspace(Vec3::K * extent, num_points)
                        .map(move |z_pt| Self::evaluate_at(params, &(x_pt + y_pt + z_pt).into()))
                })
            })
            .collect::<Vec<_>>()
            .into()
    }
}

/// Trait representing a function for which a spherical "bound" centered at the origin can be
/// reasonably defined.
pub trait EvaluateBounded: Evaluate {
    /// Give an approximate bound for the function, in the sense that the function is "sufficiently
    /// close to zero" everywhere outside a sphere whose radius is the returned value and whose
    /// center is the origin.
    fn bound(params: &Self::Parameters) -> f32;

    /// Compute a plot of the cross section of the function along a given `plane`.
    ///
    /// `num_points` points will be evaluated in a grid centered at the origin, extending to the
    /// bound of the function.
    ///
    /// For more information, see the documentation on [`GridValues`].
    fn sample_plane(
        params: &Self::Parameters,
        plane: Plane,
        num_points: usize,
    ) -> GridValues<Self::Output> {
        Self::evaluate_on_plane(params, plane, Self::bound(params), num_points)
    }

    /// Compute a plot of the function in a cube centered at the origin. `num_points` are sampled
    /// in each dimension, producing an evenly-spaced lattice of values the size of the
    /// function's bound.
    ///
    /// For more information, see [`Evaluate::evaluate_in_region`].
    fn sample_region(params: &Self::Parameters, num_points: usize) -> ComponentForm<Self::Output> {
        Self::evaluate_in_region(params, Self::bound(params), num_points)
    }
}

/// Try to find roots of a continuous function in a given `interval`. `num_initial_test` evenly
/// spaced points are used to test for sign changes (thus, zeros, by the intermediate value
/// theorem), which are then refined using Brent's method.
///
/// # Panics
/// This function will panic if the root finder does not converge.
pub fn find_roots_in_interval<'a>(
    interval: RangeInclusive<f32>,
    num_initial_tests: usize,
    f: impl Fn(f32) -> f32 + Copy + 'a,
) -> impl Iterator<Item = f32> + 'a {
    interval
        .linspace(num_initial_tests)
        .map(move |a| (a, f(a)))
        .tuple_windows()
        .filter(|((_, f_a), (_, f_b))| f_a * f_b < 0.0) // ab < 0 iff a < 0 xor b < 0.
        .map(move |((a, _), (b, _))| {
            roots::find_root_brent(a, b, f, &mut 1E-4_f32)
                .unwrap_or_else(|err| panic!("root finder encountered an error: {err}"))
        })
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::{integrate_rk4, integrate_trapezoidal, DoubleFactorial};
    use crate::geometry::Linspace;

    #[test]
    fn test_double_factorial() {
        assert_eq!(
            vec![1, 1, 2, 3, 8, 15, 48, 105, 384, 945, 3840],
            (0_u32..=10)
                .map(DoubleFactorial::double_factorial)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_trapezoidal() {
        let xs = (-2_f32..=4.0).linspace(100).collect::<Vec<_>>();

        assert_relative_eq!(
            integrate_trapezoidal(
                &xs,
                &xs.iter()
                    .map(|x| x.powi(3) - 3.0 * x * x - 2.0 * x + 4.0)
                    .collect::<Vec<_>>()
            ),
            0.0,
            epsilon = 1E-4,
        );

        assert_relative_eq!(
            integrate_trapezoidal(
                &xs,
                &xs.iter()
                    .map(|x| (x - 2.0).exp() + x.cos())
                    .collect::<Vec<_>>()
            ),
            7.523_235,
            epsilon = 5E-3,
        );
    }

    #[test]
    fn test_rk4() {
        assert_relative_eq!(integrate_rk4(|x| x, 0.0, 5.0, 1.0), 12.5, epsilon = 1E-6);

        assert_relative_eq!(
            integrate_rk4(
                |x| x.powi(3) / 3.0 - 2.0 * x + 0.5_f32.powf(x),
                -5.0,
                5.0,
                0.1,
            ),
            46.121_16,
            epsilon = 1E-4,
        );
    }

    #[test]
    fn test_rk4_triple() {
        assert_relative_eq!(
            integrate_rk4_multiple!(
                x * x * y + z.cos(),
                step: 0.1,
                x: (0., 1.),
                y: (-2., -1.),
                z: (-1., 1.),
            ),
            0.682_942,
            epsilon = 1E-6,
        );
    }
}
