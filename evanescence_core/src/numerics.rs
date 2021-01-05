use crate::geometry::Point;

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
        match q {
            0 => 1.0,
            1 => 1.0 + p as f64 - x,
            _ => {
                (((2 * q - 1 + p) as f64 - x) * associated_laguerre((q - 1, p), x)
                    - (q - 1 + p) as f64 * associated_laguerre((q - 2, p), x))
                    / q as f64
            }
        }
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
        let mut p = if m == 0 {
            1.0
        } else {
            (if m % 2 == 0 { 1.0 } else { -1.0 })  // (-1)^l
                * (2 * m - 1).multifactorial::<2>() as f64
                * (1.0 - x * x).powi(l as _).sqrt()
        };
        if l == m {
            return p;
        }

        let mut prev = p;

        // Compute `P_{m+1}^m`.
        p *= x * (2 * m + 1) as f64;
        if l - m == 1 {
            return p;
        }

        // Iteratively compute `P_{m+2}^m`, `P_{m+3}^m`, ..., `P_l^m`.
        for l in (m + 1)..l {
            (prev, p) = (
                p,
                ((2 * l + 1) as f64 * x * p - (l + m) as f64 * prev) / (l - m + 1) as f64,
            );
        }
        p
    }
}

pub trait Evaluate {
    type Output: Copy;
    type Parameters: Copy;

    fn evaluate(params: Self::Parameters, point: &Point) -> Self::Output;
}

#[macro_export]
macro_rules! assert_iterable_relative_eq {
    ($lhs:expr, $rhs: expr $(, $opt:ident = $val:expr)*) => {{
        assert_eq!($lhs.len(), $rhs.len());
        assert!(
        $lhs.iter()
            .zip($rhs.iter())
            .all(|(l, r)| approx::relative_eq!(l, r $(, $opt = $val)*)),
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
