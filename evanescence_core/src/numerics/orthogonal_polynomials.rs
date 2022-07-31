//! Implementations of the the associated Legendre functions and the associated Laguerre
//! polynomials.

use crate::numerics::double_factorial::DoubleFactorial;

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

/// See attached Mathematica notebooks for the computation of test values.
#[cfg(test)]
mod tests {
    use super::{associated_laguerre, associated_legendre};

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
        laguerre_1_0,
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
        laguerre_3_2,
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
        laguerre_4_5,
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
        laguerre_7_3,
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
        legendre_1_0,
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
        legendre_3_1,
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
        legendre_4_3,
        associated_legendre,
        (4, 3),
        &[0.0, 34.0997502740123, 0.0, -34.0997502740123, 0.0]
    );
    test!(
        legendre_4_4,
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
        legendre_5_4,
        associated_legendre,
        (5, 4),
        &[0.0, -265.781250000000, 0.0, 265.781250000000, 0.0]
    );
    test!(
        legendre_6_0,
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
