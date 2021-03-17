//! Functions and traits relating to numerical evaluation.

use std::ops::RangeInclusive;

use crate::geometry::{GridValues, Plane, Point, PointValue, Vec3};

/// Compute the `N`-th [multifactorial](https://en.wikipedia.org/wiki/Factorial#Multifactorials).
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

pub fn normalize(
    source_range: RangeInclusive<f32>,
    target_range: RangeInclusive<f32>,
    val: f32,
) -> f32 {
    (val - source_range.start()) / (source_range.end() - source_range.start())
        * (target_range.end() - target_range.start())
        + target_range.start()
}

pub mod orthogonal_polynomials {
    //! Implementations of the the associated Legendre functions and the associated Laguerre
    //! polynomials.
    use super::Multifactorial;

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
                * (2 * m - 1).multifactorial::<2>() as f32
                * (1.0 - x * x).powi(m as _).sqrt()
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

        for &col_pt in points_in_col.iter() {
            let mut row = Vec::with_capacity(num_points);
            for &row_pt in points_in_row.iter() {
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
    ) -> Vec<PointValue<Self::Output>> {
        Vec3::symmetric_linspace(Vec3::I * extent, num_points)
            .flat_map(|x_pt| {
                Vec3::symmetric_linspace(Vec3::J * extent, num_points).flat_map(move |y_pt| {
                    Vec3::symmetric_linspace(Vec3::K * extent, num_points)
                        .map(move |z_pt| Self::evaluate_at(params, &(x_pt + y_pt + z_pt).into()))
                })
            })
            .collect()
    }
}

/// Fast mathematical functions.
pub(crate) trait FastOps {
    /// The cube root function.
    fn fast_cbrt(self) -> Self;
}

impl FastOps for f32 {
    /// Fast cube root, implemented via Algorithms 5 and 6 in
    /// [Moroz et al. 2021](https://doi.org/10.3390/en14041058).
    #[allow(
        clippy::many_single_char_names, // Following paper.
        clippy::integer_division, // Intentional.
    )]
    #[inline]
    fn fast_cbrt(mut self) -> f32 {
        const K1: f32 = 1.752319676;
        const K2: f32 = 1.2509524245;
        const K3: f32 = 0.5093818292;

        if self.is_nan() {
            return self;
        }

        // The algorithm only handles positive numbers.
        let sgn = self.signum();
        self = self.abs();

        let mut i = self.to_bits();
        i = 0x548c2b4b - i / 3;
        let mut y = f32::from_bits(i);

        let mut c = self * y * y * y;
        y *= K1 - c * (K2 - K3 * c);
        let d = self * y * y;
        c = 1.0 - d * y;
        y = d * (1.0 + 0.333333333 * c);

        y * sgn
    }
}

/// Verify that two iterables containing float values are approximately equal.
#[cfg(test)]
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
    use approx::assert_relative_eq;

    use super::orthogonal_polynomials::{associated_laguerre, associated_legendre};
    use super::{FastOps, Multifactorial};

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

    #[test]
    fn test_double_factorial() {
        assert_eq!(
            vec![1, 1, 2, 3, 8, 15, 48, 105, 384, 945, 3840],
            (0_u32..=10)
                .map(Multifactorial::multifactorial::<2>)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_cbrt() {
        [
            (0.0_f32, 0.0_f32),
            (22512.2, 28.2362),
            (-83053.8, -43.6301),
            (12074.7, 22.9417),
            (95234.4, 45.6665),
            (-73976.6, -41.9789),
            (-32802.3, -32.0112),
            (-67107.9, -40.6373),
            (-48024.1, -36.3485),
            (-66945.8, -40.6045),
            (-14244.2, -24.2408),
            (-71994.7, -41.6007),
            (37898.7, 33.5899),
            (-73425.8, -41.8745),
            (70552.7, 41.321),
            (95867.7, 45.7675),
            (42809.7, 34.9822),
            (66234.2, 40.4602),
            (-23330.9, -28.5744),
            (-65244.2, -40.2576),
            (74569.8, 42.0908),
            (6709.16, 18.8606),
            (37441.6, 33.4543),
            (44715.9, 35.4939),
            (13136.1, 23.5951),
            (73729.4, 41.9321),
            (-73844.2, -41.9539),
            (15118.8, 24.7271),
            (-68574.8, -40.9312),
            (-70856., -41.3802),
            (-55508.1, -38.1463),
            (-26725.7, -29.8981),
            (-4639.62, -16.6786),
            (28716.1, 30.6226),
            (-55739.2, -38.1991),
            (91272., 45.0242),
            (-53362.1, -37.6482),
            (-41865.7, -34.7232),
            (-78111.5, -42.7469),
            (68193.2, 40.8552),
            (78626.4, 42.8407),
            (497.52, 7.92386),
            (-12492.2, -23.2031),
            (-50284.9, -36.9102),
            (20445.8, 27.3444),
            (-94989.1, -45.6273),
            (20996., 27.5875),
            (90376.3, 44.8764),
            (-16015.7, -25.2067),
            (74953.1, 42.1628),
            (-34397., -32.5217),
        ]
        .iter()
        .for_each(|&(val, expected)| {
            assert_relative_eq!(expected, val.fast_cbrt(), max_relative = 1E-4)
        });
    }

    #[test]
    fn test_cbrt_small() {
        [
            (0.377565_f32, 0.722765_f32),
            (-0.112598, -0.482884),
            (-0.0164998, -0.254581),
            (-0.236167, -0.61812),
            (-0.137506, -0.516147),
            (-0.56283, -0.825643),
            (-0.234004, -0.616228),
            (0.324958, 0.687505),
            (0.0767638, 0.424997),
            (0.289166, 0.661276),
            (-0.308712, -0.675852),
            (0.0628351, 0.397558),
            (-0.363971, -0.713985),
            (0.641814, 0.862587),
            (-0.614646, -0.85024),
            (0.0818931, 0.434259),
            (-0.0597065, -0.390847),
            (-0.047765, -0.36283),
            (0.287967, 0.66036),
            (0.148289, 0.529301),
            (-0.197911, -0.58276),
            (-0.00863066, -0.205123),
            (-0.0213903, -0.277591),
            (-0.0104858, -0.218877),
            (-0.198444, -0.583283),
            (-0.255607, -0.634636),
            (-0.699585, -0.887729),
            (0.107098, 0.474891),
            (-0.468241, -0.776527),
            (0.337018, 0.695906),
            (-0.213833, -0.597987),
            (0.393751, 0.732949),
            (-0.53413, -0.811364),
            (0.4016, 0.737787),
            (0.176495, 0.560933),
            (-0.476213, -0.780909),
            (-0.317749, -0.682383),
            (-0.236579, -0.618479),
            (-0.229639, -0.612372),
            (0.0816585, 0.433844),
            (0.0809944, 0.432665),
            (-0.153734, -0.535702),
            (-0.211659, -0.595954),
            (-0.0719213, -0.415865),
            (0.259634, 0.637951),
            (0.0132388, 0.236565),
            (-0.590054, -0.838746),
            (0.374902, 0.721062),
            (0.257372, 0.636093),
            (0.1066, 0.474153),
        ]
        .iter()
        .for_each(|&(val, expected)| {
            assert_relative_eq!(expected, val.fast_cbrt(), max_relative = 1E-4)
        });
    }
}
