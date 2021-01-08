use std::cmp::Ordering;
use std::f32::consts::{PI, SQRT_2};

use num_complex::Complex32;

use crate::{
    geometry::Point,
    numerics::{
        orthogonal_polynomials::{associated_laguerre, associated_legendre},
        Evaluate,
    },
    orbital::{LM, NL},
};

pub struct Radial;

impl Radial {
    /// Calculate the radial wavefunction normalization factor,
    /// `√( (2/n)^3 * (n-l-1)! / (2n * (n+l)!) )`.
    #[inline]
    fn normalization_factor(n: u32, l: u32) -> f32 {
        // (n-l-1)! / (n+l)! = 1 / [(n-l) * (n-l+1) * ... * (n+l-1) * (n+l)].
        let factorial_factor = (((n - l) as u64)..=((n + l) as u64)).product::<u64>() as f32;
        // Where we've taken `(2/n)^3 / 2n` out ouf the square root.
        2.0 / ((n * n) as f32 * factorial_factor.sqrt())
    }
}

impl Evaluate for Radial {
    type Output = f32;
    type Parameters = NL;

    #[inline]
    fn evaluate(NL { n, l }: NL, point: &Point) -> Self::Output {
        let rho = 2.0 * point.r() / (n as f32);
        Self::normalization_factor(n, l)
            * (-rho / 2.0).exp()
            * rho.powi(l as i32)
            * associated_laguerre((n - l - 1, 2 * l + 1), rho)
    }
}

pub struct RadialProbability;

impl Evaluate for RadialProbability {
    type Output = f32;
    type Parameters = NL;

    #[inline]
    fn evaluate(params: Self::Parameters, point: &Point) -> Self::Output {
        #[allow(non_snake_case)]
        let R = Radial::evaluate(params, point);
        R * R
    }
}


pub struct RadialProbabilityDistribution;

impl Evaluate for RadialProbabilityDensity {
    type Output = f32;
    type Parameters = NL;

    #[inline]
    fn evaluate(params: Self::Parameters, pt: &Point) -> Self::Output {
        let r = pt.r();
        #[allow(non_snake_case)]
        let R = Radial::evaluate(params, pt);
        r * r * R * R
    }
}

pub struct SphericalHarmonic;

impl SphericalHarmonic {
    /// Compute the normalization factor of a spherical harmonic, excluding the Condon-Shortley phase:
    /// `√( (2l + 1)/4pi * (l-|m|)!/(l+|m|)! )`.
    #[inline]
    fn normalization_factor(l: u32, m_abs: u32) -> f32 {
        // (l-|m|)!/(l+|m|)! = 1 / [(l-|m|+1) * (l-|m|+2) * ... * (l+|m|-1) * (l+|m|)].
        let factorial_factor =
            (((l - m_abs + 1) as u64)..=((l + m_abs) as u64)).product::<u64>() as f32;
        ((2 * l + 1) as f32 / (4.00 * PI * factorial_factor)).sqrt()
    }
}

impl Evaluate for SphericalHarmonic {
    type Output = Complex32;
    type Parameters = LM;

    #[inline]
    fn evaluate(LM { l, m }: LM, point: &Point) -> Self::Output {
        let m_abs = m.abs() as u32;
        Self::normalization_factor(l, m_abs)
            * associated_legendre((l, m_abs), point.cos_theta())
            * (Complex32::i() * m as f32 * point.phi()).exp()
    }
}

pub struct RealSphericalHarmonic;

impl Evaluate for RealSphericalHarmonic {
    type Output = f32;
    type Parameters = LM;

    #[inline]
    fn evaluate(LM { l, m }: LM, point: &Point) -> Self::Output {
        let m_abs = m.abs() as u32;
        SphericalHarmonic::normalization_factor(l, m_abs)
            * if m_abs % 2 == 0 { 1.0 } else { -1.0 } // (-1)^(-m), to cancel out the Condon-Shortley phase from `associated_legendre`.
            * associated_legendre((l, m_abs), point.cos_theta())
            * match m.cmp(&0) {
                Ordering::Greater => SQRT_2 * (m as f32 * point.phi()).cos(),
                Ordering::Equal => 1.0,
                Ordering::Less => SQRT_2 * (m_abs as f32 * point.phi()).sin(),
            }
    }
}

/// See attached Mathematica notebooks for the computation of test values.
#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;

    use super::{Radial, RealSphericalHarmonic};
    use crate::{
        assert_iterable_relative_eq,
        geometry::Point,
        numerics::Evaluate,
        orbital::{LM, NL},
    };

    lazy_static! {
        static ref TEST_POINTS: Vec<Point> = vec![
            Point::new(0.4817551668747674, -0.0804650251296536, -5.6874218288168015),
            Point::new(2.0333187824258494, -5.438747021019332, -0.6049420414492214),
            Point::new(-5.536298275253506, 4.6076805316895, -1.2262339330118561),
            Point::new(-4.149645839500807, -1.2219161435660775, 6.136358860884453),
            Point::new(0.16027731917592172, -1.03637296675495, 2.9708473002364815),
            Point::new(0.9588954657662077, -5.518368529663465, -4.652089232680136),
            Point::new(-5.511550802584331, 4.47433749034273, 2.7803369868007075),
            Point::new(-4.867929449744333, 0.5252811429346673, -8.256693767935815),
            Point::new(-0.15569673568314418, -7.415856784977347, 5.713181575221412),
            Point::new(4.391483176478295, -6.632126843233279, 2.103909081619213),
        ];
    }

    // #[test]
    // fn print_radii() {
    //     TEST_POINTS.iter().for_each(|pt| print!("{}, ", pt.r()));
    //     println!();
    // }

    // #[test]
    // fn print_theta_phi() {
    //     TEST_POINTS
    //         .iter()
    //         .for_each(|pt| println!("{{{}, {}}},", pt.cos_theta().acos(), pt.phi()));
    //     println!();
    // }

    macro_rules! test {
        ($fn_name:ident, $target_fn:expr, $target_params:expr, $expected:expr) => {
            #[test]
            fn $fn_name() {
                let calculated: Vec<_> = TEST_POINTS
                    .iter()
                    .map(|pt| $target_fn($target_params, pt))
                    .collect();
                assert_iterable_relative_eq!($expected, &calculated, max_relative = 1E-4_f32);
            }
        };
    }

    test!(
        test_radial_1_0,
        Radial::evaluate,
        NL::new(1, 0).unwrap(),
        &[
            0.00663625, 0.0058303, 0.00134232, 0.00109754, 0.0856609, 0.00137691, 0.00097705,
            0.00013556, 0.00017174, 0.00053425,
        ]
    );
    test!(
        test_radial_2_1,
        Radial::evaluate,
        NL::new(2, 1).unwrap(),
        &[
            0.06712, 0.0643393, 0.0386382, 0.0359008, 0.133092, 0.0389966, 0.0343976, 0.0161317,
            0.0177099, 0.0274495,
        ]
    );
    test!(
        test_radial_3_0,
        Radial::evaluate,
        NL::new(3, 0).unwrap(),
        &[
            -0.0224952, -0.0202023, 0.00281192, 0.00536186, -0.0491676, 0.00247796, 0.00675915,
            0.0223802, 0.0212521, 0.0131223
        ]
    );
    test!(
        test_radial_5_3,
        Radial::evaluate,
        NL::new(5, 3).unwrap(),
        &[
            0.00865692, 0.00894101, 0.0117124, 0.0120124, 0.00286186, 0.0116729, 0.0121757,
            0.0137583, 0.013689, 0.0129009,
        ]
    );

    test!(
        test_real_sph_harm_1_0,
        RealSphericalHarmonic::evaluate,
        LM::new(1, 0).unwrap(),
        &[
            -0.486811, -0.0506311, -0.0820011, 0.399348, 0.46074, -0.312183, 0.178182, -0.420266,
            0.298149, 0.124939
        ]
    );
    test!(
        test_real_sph_harm_1_1,
        RealSphericalHarmonic::evaluate,
        LM::new(1, 1).unwrap(),
        &[
            0.0412355, 0.17018, -0.370225, -0.270055, 0.0248569, 0.0643476, -0.353216, -0.247778,
            -0.0081252, 0.260785
        ]
    );
    test!(
        test_real_sph_harm_1_n1,
        RealSphericalHarmonic::evaluate,
        LM::new(1, -1).unwrap(),
        &[
            -0.0068873, -0.455201, 0.308126, -0.0795211, -0.160728, -0.370316, 0.286744, 0.0267368,
            -0.387006, -0.393845
        ]
    );
    test!(
        test_real_sph_harm_2_1,
        RealSphericalHarmonic::evaluate,
        LM::new(2, 1).unwrap(),
        &[
            -0.0918672, -0.0394327, 0.138936, -0.493553, 0.0524122, -0.0919329, -0.288027,
            0.476559, -0.0110866, 0.149112
        ]
    );
    test!(
        test_real_sph_harm_3_n2,
        RealSphericalHarmonic::evaluate,
        LM::new(3, -2).unwrap(),
        &[
            0.00342614, 0.0971969, 0.231812, 0.212525, -0.045616, 0.184347, -0.44722, 0.0689953,
            0.0232332, -0.318002
        ]
    );
    test!(
        test_real_sph_harm_5_n3,
        RealSphericalHarmonic::evaluate,
        LM::new(5, -3).unwrap(),
        &[
            -0.0011583, -0.207523, -0.305109, -0.355185, 0.113205, 0.517974, 0.0691625, 0.11642,
            0.570845, 0.0332508
        ]
    );
}
