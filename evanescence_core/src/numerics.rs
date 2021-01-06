use std::iter;

use getset::Getters;

use crate::geometry::{Point, Vec3};

pub trait Multifactorial {
    fn multifactorial<const N: u8>(self) -> Self;
}

macro_rules! impl_multifactorial {
    ($($T:ty),+) => {
        $(impl Multifactorial for $T {
            #[inline]
            fn multifactorial<const N: u8>(self) -> Self {
                if self <= 1 {
                    return 1;
                }
                let mut acc = self;
                let delta = N as $T;
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
impl_multifactorial!(u8, u16, u32, u64, usize);

pub mod orthogonal_polynomials {
    use super::Multifactorial;

    /// The associated Laguerre polynomials, `L_{q}^{p}(x)`.
    ///
    /// Implemented via recurrence relation:
    /// <https://en.wikipedia.org/wiki/Laguerre_polynomials#Generalized_Laguerre_polynomials>.
    #[inline]
    pub fn associated_laguerre((q, p): (u32, u32), x: f64) -> f64 {
        if q == 0 {
            return 1.0;
        }

        #[allow(non_snake_case)]
        let mut L = 1.0 + p as f64 - x;
        if q == 1 {
            return L;
        }

        let mut prev = 1.0;
        for q in 1..q {
            (prev, L) = (
                L,
                (((2 * q + 1 + p) as f64 - x) * L - (q + p) as f64 * prev) / (q + 1) as f64,
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
    pub fn associated_legendre((l, m): (u32, u32), x: f64) -> f64 {
        // Check for special cases.
        if m > l {
            return 0.0;
        };

        // Compute `P_m^m`.
        #[allow(non_snake_case)]
        let mut P = if m == 0 {
            1.0
        } else {
            (if m % 2 == 0 { 1.0 } else { -1.0 })  // (-1)^l
                * (2 * m - 1).multifactorial::<2>() as f64
                * (1.0 - x * x).powi(l as _).sqrt()
        };
        if l == m {
            return P;
        }

        let mut prev = P;

        // Compute `P_{m+1}^m`.
        P *= x * (2 * m + 1) as f64;
        if l - m == 1 {
            return P;
        }

        // Iteratively compute `P_{m+2}^m`, `P_{m+3}^m`, ..., `P_l^m`.
        for l in (m + 1)..l {
            (prev, P) = (
                P,
                ((2 * l + 1) as f64 * x * P - (l + m) as f64 * prev) / (l - m + 1) as f64,
            );
        }
        P
    }
}

/// A point and the value of a function evaluated at that point.
pub type Evaluation<T> = (Point, T);

/// Type storing a collection of evaluations, where values in each dimension (x, y, z, and value)
/// is stored in a separate vector. Each index, across the four vectors, corresponds to
/// a single point and its associated value.
///
/// It may be thought of as the transpose of `Vec<Evaluation<T>>`.
///
/// This type cannot be manually constructed and should instead be obtained from a
/// [`Vec<Evaluation<T>>`] via conversion traits.
///
/// # Safety
/// All four vectors must be the same length.
#[derive(Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct ComponentForm<T> {
    /// List of x-values.
    xs: Vec<f64>,
    /// List of y-values.
    ys: Vec<f64>,
    /// List of z-values.
    zs: Vec<f64>,
    /// List of wavefunction values evaluated at the corresponding (x, y, z) coordinate.
    vals: Vec<T>,
}

impl<T> ComponentForm<T> {
    /// Decompose `Self` into a four-tuple of its inner vectors,
    /// in the order (x, y, z, value).
    pub fn into_components(self) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<T>) {
        (self.xs, self.ys, self.zs, self.vals)
    }
}

impl<T> From<Vec<Evaluation<T>>> for ComponentForm<T> {
    fn from(v: Vec<Evaluation<T>>) -> Self {
        let len = v.len();
        let (mut xs, mut ys, mut zs, mut vals) = (
            Vec::with_capacity(len),
            Vec::with_capacity(len),
            Vec::with_capacity(len),
            Vec::with_capacity(len),
        );
        v.into_iter().for_each(|(pt, val)| {
            xs.push(pt.x());
            ys.push(pt.y());
            zs.push(pt.z());
            vals.push(val);
        });
        ComponentForm { xs, ys, zs, vals }
    }
}

pub trait Evaluate {
    type Output: Copy;
    type Parameters: Copy;

    /// Evaluate `Self` at a certain point, returning the value only.
    fn evaluate(params: Self::Parameters, point: &Point) -> Self::Output;

    /// Evaluate `Self` at a certain point, returning the point *and* the value in the form of an
    /// [`Evaluation`], or a `(Point, Self::Output)`.
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn evaluate_at(params: Self::Parameters, point: &Point) -> Evaluation<Self::Output> {
        (*point, Self::evaluate(params, point))
    }

    /// Evaluate `Self` on a line segment running from `begin` to `end` at a total of `num_points`
    /// different points, all evenly spaced (à la "`linspace`" operation).
    fn evaluate_on_line_segment(
        params: Self::Parameters,
        begin: Vec3,
        end: Vec3,
        num_points: usize,
    ) -> Vec<Evaluation<Self::Output>> {
        Vec3::linspace(begin, end, num_points)
            .map(|pt| Self::evaluate_at(params, &pt.into()))
            .collect()
    }

    /// Evaluate `Self` on a grid of points evenly covering a parallelogram. This parallelogram
    /// is defined as follows:
    ///
    /// * Its center is the origin;
    /// * The midpoint of its "top" edge is defined by `extent_horizontal`; and
    /// * The the midpoint of its "right" edge is defined by `extent_vertical`.
    ///
    /// Consider calling `evaluate_on_plane` with the following arguments:
    ///
    /// ```ignore
    /// let points = SomeEvaluator::evaluate_on_plane(
    ///     some_params,
    ///     (X, 3),
    ///     (Y, 5),
    /// );
    /// ```
    ///
    /// Then, these are the points where evaluation will occur (with order annotated, and where
    /// `O` indicates the origin):
    ///
    /// ```text
    ///           1      2      3
    ///          x------X------x
    ///         /             /
    ///        x 4    x 5    x 6
    ///       /             /
    ///      x 7    O 8    Y 9
    ///     /             /
    ///    x 10   x 11   x 12
    ///   /             /
    ///  x------x------x
    /// 13     14     15
    /// ```
    fn evaluate_on_plane(
        params: Self::Parameters,
        (extent_horizontal, num_pts_horizontal): (Vec3, usize),
        (extent_vertical, num_pts_vertical): (Vec3, usize),
    ) -> Vec<Evaluation<Self::Output>> {
        let vertical_linspace = Vec3::linspace(extent_vertical, -extent_vertical, num_pts_vertical);
        let horizontal_linspace =
            Vec3::linspace(-extent_horizontal, extent_horizontal, num_pts_horizontal)
                .collect::<Vec<_>>();
        let horizontal_linspace_copies =
            iter::repeat(horizontal_linspace).take(vertical_linspace.len());
        vertical_linspace
            .zip(horizontal_linspace_copies)
            .flat_map(|(vertical_pt, horizontal_linspace)| {
                horizontal_linspace
                    .into_iter()
                    .map(move |horizontal_pt| horizontal_pt + vertical_pt)
            })
            .map(|pt| Self::evaluate_at(params, &pt.into()))
            .collect()
    }
}

#[macro_export]
macro_rules! assert_iterable_relative_eq {
    ($lhs:expr, $rhs: expr $(, $opt:ident = $val:expr)*) => {{
        assert_eq!($lhs.len(), $rhs.len());
        assert!(
            $lhs.iter()
                .zip($rhs.iter())
                .all(|(l, r)| approx::relative_eq!(l, r $(, $opt = $val)*)
        ),
        indoc::indoc! {"
            assertion failed: `(left ≈ right)`
            left: `{:?}`
            right: `{:?}`
        "},
        $lhs,
        $rhs
    );
    }};
    ($lhs:expr, $rhs: expr $(, $opt:ident = $val:expr)*,) => {
        assert_iterable_relative_eq!($lhs, $rhs, $(, $opt = $val)*)
    }
}

/// See attached Mathematica notebooks for the computation of test values.
#[cfg(test)]
mod tests {
    use super::orthogonal_polynomials::{associated_laguerre, associated_legendre};
    use super::Multifactorial;
    use crate::assert_iterable_relative_eq;

    macro_rules! test {
        ($fn_name:ident, $target_fn:ident, $target_params:expr, $expected:expr) => {
            #[test]
            fn $fn_name() {
                let calculated: Vec<f64> = (-2..=2)
                    .map(|x| $target_fn($target_params, x as f64 / 2.0))
                    .collect();
                assert_iterable_relative_eq!($expected, &calculated, max_relative = 1E-10_f64);
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

    #[test]
    fn test_double_factorial() {
        assert_eq!(
            vec![1, 1, 2, 3, 8, 15, 48, 105, 384, 945, 3840],
            (0_u32..=10)
                .map(Multifactorial::multifactorial::<2>)
                .collect::<Vec<_>>()
        );
    }
}
