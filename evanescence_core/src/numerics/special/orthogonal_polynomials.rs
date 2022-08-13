//! Implementations of the the associated Legendre functions and the associated Laguerre
//! polynomials.

use std::f32::consts::PI;

use super::binomial_coefficient;
use crate::numerics::polynomial::Polynomial;

/// The associated Laguerre polynomials, `L_{n}^{a}(x)`.
pub fn associated_laguerre(n: u32, a: u32) -> Polynomial {
    (0..=n)
        .map(|i| {
            let mut a_i = (-1_f32).powi(i as i32) * binomial_coefficient(n + a, n - i) as f32;
            (1..=i).for_each(|j| a_i /= j as f32);
            a_i
        })
        .collect()
}

/// The associated Legendre polynomials `P_{l}^{m}(x)`, renormalized by the spherical harmonics
/// normalization factor `√( (2l + 1)/4pi * (l-m)!/(l+m)! )`.
///
/// Note that the Condon-Shortley phase is _not_ included!
///
/// Ref. _Numerical Recipes_ 3rd ed., section 6.7.
pub fn renormalized_associated_legendre((l, m): (u32, u32), x: f32) -> f32 {
    if m > l {
        return 0.;
    };

    let m_f32 = m as f32;

    let mut p_mm = 1.;

    if m != 0 {
        let one_minus_x_sq = (1. - x) * (1. + x);
        let mut double_factorial_factor = 1.;
        for _ in 0..m {
            p_mm *= one_minus_x_sq * double_factorial_factor / (double_factorial_factor + 1.);
            double_factorial_factor += 2.;
        }
    }

    p_mm = ((2. * m_f32 + 1.) * p_mm / (4. * PI)).sqrt();

    if l == m {
        return p_mm;
    }

    let sqrt_2mp3 = (2. * m_f32 + 3.).sqrt();

    let mut p_m_mp1 = x * sqrt_2mp3 * p_mm;
    if l == m + 1 {
        return p_m_mp1;
    }

    let mut p_ll = p_m_mp1;
    let mut old_factor = sqrt_2mp3;
    for l_step in (m + 2)..=l {
        let l_step = l_step as f32;
        let factor = ((4. * l_step * l_step - 1.) / (l_step * l_step - m_f32 * m_f32)).sqrt();
        p_ll = (x * p_m_mp1 - p_mm / old_factor) * factor;
        old_factor = factor;
        (p_mm, p_m_mp1) = (p_m_mp1, p_ll);
    }
    p_ll
}

/// See attached Mathematica notebooks for the computation of test values.
#[cfg(test)]
mod tests {
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
}