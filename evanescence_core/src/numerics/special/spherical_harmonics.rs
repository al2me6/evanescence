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
        let (l, m) = (self.lm.l(), self.lm.m());
        let y = renormalized_associated_legendre((l, m.unsigned_abs()), point.cos_theta())
            * (Complex32::i() * m as f32 * point.phi()).exp();
        // Condon-Shortley phase.
        if m & 0x1 == 0 {
            y
        } else {
            -y
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
    use std::sync::LazyLock;

    use na::vector;

    use super::RealSphericalHarmonic;
    use crate::geometry::point::SphericalPoint3;
    use crate::numerics::Function;
    use crate::orbital::quantum_numbers::Lm;

    static POINTS: LazyLock<Vec<SphericalPoint3>> = LazyLock::new(|| {
        vec![
            SphericalPoint3::from(vector![
                0.4817551668747674,
                -0.0804650251296536,
                -5.6874218288168015
            ]),
            SphericalPoint3::from(vector![
                2.0333187824258494,
                -5.438747021019332,
                -0.6049420414492214
            ]),
            SphericalPoint3::from(vector![
                -5.536298275253506,
                4.6076805316895,
                -1.2262339330118561
            ]),
            SphericalPoint3::from(vector![
                -4.149645839500807,
                -1.2219161435660775,
                6.136358860884453
            ]),
            SphericalPoint3::from(vector![
                0.16027731917592172,
                -1.03637296675495,
                2.9708473002364815
            ]),
            SphericalPoint3::from(vector![
                0.9588954657662077,
                -5.518368529663465,
                -4.652089232680136
            ]),
            SphericalPoint3::from(vector![
                -5.511550802584331,
                4.47433749034273,
                2.7803369868007075
            ]),
            SphericalPoint3::from(vector![
                -4.867929449744333,
                0.5252811429346673,
                -8.256693767935815
            ]),
            SphericalPoint3::from(vector![
                -0.15569673568314418,
                -7.415856784977347,
                5.713181575221412
            ]),
            SphericalPoint3::from(vector![
                4.391483176478295,
                -6.632126843233279,
                2.103909081619213
            ]),
        ]
    });

    macro_rules! test {
        ($fn_name:ident, $target:ty, $target_params:expr, $expected:expr) => {
            #[test]
            fn $fn_name() {
                let evaluator = <$target>::new($target_params);
                let calculated: Vec<_> = POINTS.iter().map(|pt| evaluator.evaluate(pt)).collect();
                assert_iterable_approx_eq!($expected, &calculated, max_relative = 5E-5);
            }
        };
    }

    test!(
        real_sph_1_0,
        RealSphericalHarmonic,
        Lm::new(1, 0).unwrap(),
        &[
            -0.486811, -0.0506311, -0.0820011, 0.399348, 0.46074, -0.312183, 0.178182, -0.420266,
            0.298149, 0.124939
        ]
    );
    test!(
        real_sph_1_1,
        RealSphericalHarmonic,
        Lm::new(1, 1).unwrap(),
        &[
            0.0412355, 0.17018, -0.370225, -0.270055, 0.0248569, 0.0643476, -0.353216, -0.247778,
            -0.0081252, 0.260785
        ]
    );
    test!(
        real_sph_1_n1,
        RealSphericalHarmonic,
        Lm::new(1, -1).unwrap(),
        &[
            -0.0068873, -0.455201, 0.308126, -0.0795211, -0.160728, -0.370316, 0.286744, 0.0267368,
            -0.387006, -0.393845
        ]
    );
    test!(
        real_sph_2_1,
        RealSphericalHarmonic,
        Lm::new(2, 1).unwrap(),
        &[
            -0.0918672, -0.0394327, 0.138936, -0.493553, 0.0524122, -0.0919329, -0.288027,
            0.476559, -0.0110866, 0.149112
        ]
    );
    test!(
        real_sph_3_n2,
        RealSphericalHarmonic,
        Lm::new(3, -2).unwrap(),
        &[
            0.00342614, 0.0971969, 0.231812, 0.212525, -0.045616, 0.184347, -0.44722, 0.0689953,
            0.0232332, -0.318002
        ]
    );
    test!(
        real_sph_5_n3,
        RealSphericalHarmonic,
        Lm::new(5, -3).unwrap(),
        &[
            -0.0011583, -0.207523, -0.305109, -0.355185, 0.113205, 0.517974, 0.0691625, 0.11642,
            0.570845, 0.0332508
        ]
    );
}
