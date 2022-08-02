//! Implementations of the the associated Legendre functions and the associated Laguerre
//! polynomials.

use super::polynomials::Polynomial;
use crate::numerics::double_factorial::DoubleFactorial;

/// The associated Laguerre polynomials, `L_{n}^{a}(x)`.
pub fn associated_laguerre(n: u32, a: u32) -> Polynomial {
    (0..=n)
        .map(|i| {
            let mut a_i =
                (-1_f32).powi(i as i32) * super::binomial_coefficient(n + a, n - i) as f32;
            (1..=i).for_each(|j| a_i /= j as f32);
            a_i
        })
        .collect()
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
    use super::associated_legendre;

    #[test]
    fn associated_laguerre() {
        #[derive(serde::Deserialize)]
        struct TestCase {
            n: u32,
            a: u32,
            coeffs: Vec<f64>,
        }

        let json = std::fs::read_to_string(
            [env!("CARGO_MANIFEST_DIR"), "mathematica", "laguerre.json"]
                .iter()
                .collect::<std::path::PathBuf>(),
        )
        .unwrap();
        let data: Vec<TestCase> = serde_json::from_str(&json).unwrap();

        for TestCase { n, a, coeffs } in data {
            #[allow(clippy::cast_possible_truncation)] // Intentional.
            let expected = coeffs.into_iter().map(|a_i| a_i as f32).collect::<Vec<_>>();
            let computed = super::associated_laguerre(n, a);
            assert_iterable_approx_eq!(ulps_eq, expected, computed, max_ulps = 1);
        }
    }

    macro_rules! test {
        ($fn_name:ident, $target_fn:ident, $target_params:expr, $expected:expr) => {
            #[test]
            fn $fn_name() {
                let calculated: Vec<f32> = (-2..=2)
                    .map(|x| $target_fn($target_params, x as f32 / 2.0))
                    .collect();
                assert_iterable_approx_eq!(ulps_eq, $expected, &calculated, max_ulps = 1);
            }
        };
    }

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
