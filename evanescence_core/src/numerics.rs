//! Functions and traits relating to numerical evaluation.

use std::ops::RangeInclusive;

use crate::geometry::{ComponentForm, GridValues, Plane, Point, PointValue, Vec3};

/// Compute the [double factorial](https://en.wikipedia.org/wiki/Double_factorial).
pub trait DoubleFactorial {
    /// `x!!`
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

/// Implementations of the the associated Legendre functions and the associated Laguerre
/// polynomials.
pub mod orthogonal_polynomials {
    use super::DoubleFactorial;

    /// The associated Laguerre polynomials, `L_{q}^{p}(x)`.
    ///
    /// Implemented via recurrence relation:
    /// <https://en.wikipedia.org/wiki/Laguerre_polynomials#Generalized_Laguerre_polynomials>.
    #[inline]
    pub fn associated_laguerre((q, p): (u32, u32), x: f32) -> f32 {
        if q == 0 {
            return 1.0;
        }

        #[allow(non_snake_case)]
        let mut L = 1.0 + p as f32 - x;
        if q == 1 {
            return L;
        }

        let mut prev = 1.0;
        for q in 1..q {
            (prev, L) = (
                L,
                (((2 * q + 1 + p) as f32 - x) * L - (q + p) as f32 * prev) / (q + 1) as f32,
            );
        }
        L
    }

    /// The associated Legendre functions, `P_{l}^{m}(x)`, implemented for nonnegative
    /// values of `l` and `m` only.
    ///
    /// Note that the Condon-Shortley phase is **included**.
    ///
    /// Implemented via recurrence relation:
    /// <https://en.wikipedia.org/wiki/Associated_Legendre_polynomials#Recurrence_formula>.
    #[inline]
    pub fn associated_legendre((l, m): (u32, u32), x: f32) -> f32 {
        // Check for special cases.
        if m > l {
            return 0.0;
        };

        // Compute P_m^m.
        #[allow(non_snake_case)]
        let mut P = if m == 0 {
            1.0 // Since m <= l, this is P_0^0(x) = 1.
        } else {
            // P_m^m(x) = (-1)^l (2m - 1)!! (1 - x^2)^(m/2).
            (if m % 2 == 0 { 1.0 } else { -1.0 })  // (-1)^l
                * (2 * m - 1).double_factorial() as f32
                * (1.0 - x * x).powi(m as i32).sqrt()
        };
        if l == m {
            return P;
        }

        let mut prev = P;

        // Compute P_{m+1}^m(x) = x (2m + 1) P_m^m(x).
        P *= x * (2 * m + 1) as f32;
        if l - m == 1 {
            return P;
        }

        // Iteratively compute P_{m+2}^m, P_{m+3}^m, ..., P_l^m.
        // (k - m + 1) P_{k+1}^m(x) = (2k + 1) x P_k^m(x) - (k + m) P_{k-1}^m(x).
        for k in (m + 1)..l {
            (prev, P) = (
                P,
                ((2 * k + 1) as f32 * x * P - (k + m) as f32 * prev) / (k - m + 1) as f32,
            );
        }
        P
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
        Vec3::linspace(range, num_points)
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

/// See attached Mathematica notebooks for the computation of test values.
#[cfg(test)]
mod tests {
    use super::orthogonal_polynomials::{associated_laguerre, associated_legendre};

    macro_rules! test {
        ($fn_name:ident, $target_fn:ident, $target_params:expr, $expected:expr) => {
            #[test]
            fn $fn_name() {
                let calculated: Vec<f32> = (-2..=2)
                    .map(|x| $target_fn($target_params, x as f32 / 2.0))
                    .collect();
                assert_iterable_relative_eq!($expected, &calculated, max_relative = 1E-6_f32);
            }
        };
    }
    test!(
        test_laguerre_1_0,
        associated_laguerre,
        (1, 0),
        &[
            2.00000000000000,
            1.50000000000000,
            1.00000000000000,
            0.500000000000000,
            0.0
        ]
    );
    test!(
        test_laguerre_3_2,
        associated_laguerre,
        (3, 2),
        &[
            22.6666666666667,
            15.6458333333333,
            10.0000000000000,
            5.60416666666667,
            2.33333333333333
        ]
    );
    test!(
        test_laguerre_4_5,
        associated_laguerre,
        (4, 5),
        &[
            229.541666666667,
            172.690104166667,
            126.000000000000,
            88.3151041666667,
            58.5416666666667
        ]
    );
    test!(
        test_laguerre_7_3,
        associated_laguerre,
        (7, 3),
        &[
            496.389087301587,
            261.199437313988,
            120.000000000000,
            42.4259967137897,
            5.63869047619048
        ]
    );

    test!(
        test_legendre_1_0,
        associated_legendre,
        (1, 0),
        &[
            -1.00000000000000,
            -0.500000000000000,
            0.0,
            0.500000000000000,
            1.00000000000000
        ]
    );
    test!(
        test_legendre_3_1,
        associated_legendre,
        (3, 1),
        &[
            0.0,
            -0.324759526419164,
            1.50000000000000,
            -0.324759526419164,
            0.0
        ]
    );
    test!(
        test_legendre_4_3,
        associated_legendre,
        (4, 3),
        &[0.0, 34.0997502740123, 0.0, -34.0997502740123, 0.0]
    );
    test!(
        test_legendre_4_4,
        associated_legendre,
        (4, 4),
        &[
            0.0,
            59.0625000000000,
            105.000000000000,
            59.0625000000000,
            0.0
        ]
    );
    test!(
        test_legendre_5_4,
        associated_legendre,
        (5, 4),
        &[0.0, -265.781250000000, 0.0, 265.781250000000, 0.0]
    );
    test!(
        test_legendre_6_0,
        associated_legendre,
        (6, 0),
        &[
            1.00000000000000,
            0.323242187500000,
            -0.312500000000000,
            0.323242187500000,
            1.00000000000000
        ]
    );
}
