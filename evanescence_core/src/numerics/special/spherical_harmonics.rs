use std::cmp::Ordering;
use std::f32::consts::SQRT_2;

use num::complex::Complex32;

use super::orthogonal_polynomials::renormalized_associated_legendre;
use crate::geometry::point::{SphericalCoordinatesExt, SphericalPoint3};
use crate::numerics::Function;
use crate::orbital::quantum_numbers::Lm;
use crate::utils::sup_sub_string::SupSubString;

/// Implementation of the spherical harmonics, `Y_l^m(θ,φ)`, including the Condon-Shortley phase.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SphericalHarmonic {
    lm: Lm,
}

impl SphericalHarmonic {
    pub fn new(lm: Lm) -> Self {
        Self { lm }
    }
}

impl Function<3, SphericalPoint3> for SphericalHarmonic {
    type Output = Complex32;

    #[inline]
    fn evaluate(&self, point: &SphericalPoint3) -> Self::Output {
        /// `(-1)^m * x`
        #[inline]
        fn times_n1_pow_m(x: Complex32, m: u32) -> Complex32 {
            if m & 0x1 == 0 {
                x
            } else {
                -x
            }
        }

        let (l, m_abs) = (self.lm.l(), self.lm.m().unsigned_abs());

        // `Y_l^|m|` with Condon-Shortley phase added.
        let y_l_m_abs = times_n1_pow_m(
            renormalized_associated_legendre((l, m_abs), point.cos_theta())
                * (Complex32::i() * m_abs as f32 * point.phi()).exp(),
            m_abs,
        );

        // `Y_l^{-m} = (-1)^m Y_l^|m|*`
        if self.lm.m() >= 0 {
            y_l_m_abs
        } else {
            times_n1_pow_m(y_l_m_abs.conj(), m_abs)
        }
    }
}

/// Implementation of the real spherical harmonics, `S_lm(θ,φ)`.
///
/// See [Blanco et al. 1997](https://doi.org/10.1016/S0166-1280(97)00185-1) for more information.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RealSphericalHarmonic {
    lm: Lm,
}

impl RealSphericalHarmonic {
    pub fn new(lm: Lm) -> Self {
        Self { lm }
    }
}

impl Function<3, SphericalPoint3> for RealSphericalHarmonic {
    type Output = f32;

    #[inline]
    fn evaluate(&self, point: &SphericalPoint3) -> Self::Output {
        let (l, m) = (self.lm.l(), self.lm.m());
        let m_abs = m.unsigned_abs();
        renormalized_associated_legendre((l, m_abs), point.cos_theta())
            * match m.cmp(&0) {
                Ordering::Greater => SQRT_2 * (m as f32 * point.phi()).cos(),
                Ordering::Equal => 1.0,
                Ordering::Less => SQRT_2 * (m_abs as f32 * point.phi()).sin(),
            }
    }
}

impl RealSphericalHarmonic {
    /// Give the (abbreviated) Cartesian expression for the linear combination that corresponds
    /// to a certain set of `l` and `m` quantum numbers.
    ///
    /// This is only implemented for `l` up to 4 and returns `None` for larger values.
    ///
    /// See <https://en.wikipedia.org/wiki/Table_of_spherical_harmonics#Real_spherical_harmonics>.
    pub fn cartesian_expression(lm: Lm) -> Option<SupSubString> {
        use crate::sup_sub_string as s;
        let (l, m) = (lm.l(), lm.m());
        match l {
            0 => Some(s![""]), // s orbital.
            1 => Some(match m {
                // p orbitals.
                -1 => s!["y"],
                0 => s!["z"],
                1 => s!["x"],
                _ => unreachable!(),
            }),
            2 => Some(match m {
                // d orbitals.
                -2 => s!["xy"],
                -1 => s!["yz"],
                0 => s!["z" sup("2")],
                1 => s!["xz"],
                2 => s!["x" sup("2") "−y" sup("2")],
                _ => unreachable!(),
            }),
            3 => Some(match m {
                // f orbitals.
                -3 => s!["y(3x" sup("2") "−y" sup("2") ")"],
                -2 => s!["xyz"],
                -1 => s!["yz" sup("2")],
                0 => s!["z" sup("3")],
                1 => s!["xz" sup("2")],
                2 => s!["z(x" sup("2") "−y" sup("2") ")"],
                3 => s!["x(x" sup("2") "−3y" sup("2") ")"],
                _ => unreachable!(),
            }),
            4 => Some(match m {
                // g orbitals.
                -4 => s!["xy(x" sup("2") "−y" sup("2") ")"],
                -3 => s!["y" sup("3") "z"],
                -2 => s!["xyz" sup("2")],
                -1 => s!["yz" sup("3")],
                0 => s!["z" sup("4")],
                1 => s!["xz" sup("3")],
                2 => s!["z" sup("2") "(x" sup("2") "−y" sup("2") ")"],
                3 => s!["x" sup("3") "z"],
                4 => s!["x" sup("4") "+y" sup("4")],
                _ => unreachable!(),
            }),
            5 => Some(match m {
                // h orbitals.
                -5 => s!["x" sup("4") "y"],
                -4 => s!["z(4x" sup("3") "y−4xy" sup("3") ")"],
                -3 => s!["y" sup("3") "z" sup("2")],
                -2 => s!["xyz" sup("3")],
                -1 => s!["yz" sup("4")],
                0 => s!["z" sup("5")],
                1 => s!["xz" sup("4")],
                2 => s!["z" sup("3") "(x" sup("2") "−y" sup("2") ")"],
                3 => s!["x" sup("3") "z" sup("2")],
                4 => s!["z(x" sup("4") "−6x" sup("2") "y" sup("2") "+y" sup("4") ")"],
                5 => s!["xy" sup("4")],
                _ => unreachable!(),
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {

    use approx::assert_relative_eq;
    use num::complex::Complex32;
    use serde::Deserialize;

    use super::{RealSphericalHarmonic, SphericalHarmonic};
    use crate::geometry::point::SphericalPoint3;
    use crate::numerics::Function;
    use crate::orbital::quantum_numbers::Lm;

    #[derive(Deserialize)]
    struct TestCase<T> {
        l: u32,
        m: i32,
        samples: Vec<T>,
    }

    #[test]
    fn real_spherical_harmonics() {
        #[derive(Deserialize)]
        struct Sample {
            theta: f32,
            phi: f32,
            val: f32,
        }

        let data: Vec<TestCase<Sample>> = crate::utils::load_test_cases("real_spherical_harmonics");
        for TestCase { l, m, samples } in data {
            let real_sph = RealSphericalHarmonic::new(Lm::new(l, m).unwrap());
            for Sample {
                theta,
                phi,
                val: expected,
            } in samples
            {
                let computed = real_sph.evaluate(&SphericalPoint3::new_spherical(1.0, theta, phi));
                assert_relative_eq!(expected, computed, max_relative = 5E-4);
            }
        }
    }

    #[test]
    fn complex_spherical_harmonics() {
        #[derive(Deserialize)]
        struct Sample {
            theta: f32,
            phi: f32,
            val: (f32, f32),
        }

        let data: Vec<TestCase<Sample>> =
            crate::utils::load_test_cases("complex_spherical_harmonics");
        for TestCase { l, m, samples } in data {
            let sph = SphericalHarmonic::new(Lm::new(l, m).unwrap());
            for Sample {
                theta,
                phi,
                val: (expected_re, expected_im),
            } in samples
            {
                let expected = Complex32::new(expected_re, expected_im);
                let computed = sph.evaluate(&SphericalPoint3::new_spherical(1.0, theta, phi));
                assert_relative_eq!(expected, computed, max_relative = 5E-3);
            }
        }
    }
}
