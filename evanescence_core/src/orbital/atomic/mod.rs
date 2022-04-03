//! This module implements real and complex atomic orbitals, including their radial and
//! angular components. Also included are probability and probability distribution functions
//! for the radial wavefunction.

use crate::geometry::{Linspace, Point};
use crate::numerics::orthogonal_polynomials::associated_laguerre;
use crate::numerics::{self, Evaluate};
use crate::orbital::quantum_numbers::{Nl, Qn};

pub mod complex;
pub mod real;

/// Implementation of the radial component of the hydrogenic wavefunction.
pub struct Radial {
    nl: Nl,
    normalization: f32,
}

impl Radial {
    pub fn new(nl: Nl) -> Self {
        Self {
            nl,
            normalization: Self::normalization_factor(nl.n(), nl.l()),
        }
    }

    /// Calculate the radial wavefunction normalization factor,
    /// `√( (2Z/n)^3 * (n-l-1)! / (2n * (n+l)!) )`.
    #[inline]
    fn normalization_factor(n: u32, l: u32) -> f32 {
        // (n-l-1)! / (n+l)! = 1 / [(n-l) * (n-l+1) * ... * (n+l-1) * (n+l)].
        let factorial_factor = ((n - l)..=(n + l)).map(|k| k as f32).product::<f32>();
        // Where we've taken `(2Z/n)^3 / 2n` out ouf the square root.
        2.0 / factorial_factor.sqrt() / ((n * n) as f32)
    }

    /// Give the value of the radial wavefunction at `r` for a given `Nl`.
    #[inline]
    pub fn evaluate_r(&self, r: f32) -> f32 {
        let (n, l) = (self.nl.n(), self.nl.l());
        let rho = 2.0 * r / (n as f32);
        self.normalization
            * (-rho / 2.0).exp()
            * rho.powi(l as i32)
            * associated_laguerre((n - l - 1, 2 * l + 1), rho)
    }
}

impl Evaluate for Radial {
    type Output = f32;

    #[inline]
    fn evaluate(&self, point: &Point) -> Self::Output {
        self.evaluate_r(point.r())
    }
}

/// The radial probability distribution, `r^2R^2`.
pub struct RadialProbabilityDistribution(Radial);

impl RadialProbabilityDistribution {
    pub fn new(nl: Nl) -> Self {
        Self(Radial::new(nl))
    }

    /// Give the value of the radial probability distribution at `r` for a given `Nl`.
    #[inline]
    pub fn evaluate_r(&self, r: f32) -> f32 {
        #[allow(non_snake_case)] // Mathematical convention.
        let R = self.0.evaluate_r(r);
        r * r * R * R
    }
}

impl Evaluate for RadialProbabilityDistribution {
    type Output = f32;

    #[inline]
    fn evaluate(&self, point: &Point) -> Self::Output {
        self.evaluate_r(point.r())
    }
}

/// A radially symmetrical property associated with an orbital.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RadialPlot {
    Wavefunction,
    ProbabilityDistribution,
}

impl RadialPlot {
    /// Compute a plot of a property of an orbital's radial wavefunction (see [`RadialPlot`]).
    ///
    /// The property will be evaluated at `num_points` points evenly spaced between the origin
    /// and the maximum extent of the orbital, which is automatically estimated.
    ///
    /// The result is returned as a 2-tuple of `Vec`s, the first containing the radial points,
    /// and the second containing the values associated with the radial points.
    pub fn sample(self, qn: Qn, num_points: usize) -> (Vec<f32>, Vec<f32>) {
        let nl = Nl::from(qn);
        let rs = (0_f32..=bound(qn)).linspace(num_points).collect::<Vec<_>>();

        let vals = match self {
            Self::Wavefunction => {
                let psi = Radial::new(nl);
                rs.iter().map(|&r| psi.evaluate_r(r)).collect()
            }
            Self::ProbabilityDistribution => {
                let psi_sq = RadialProbabilityDistribution::new(nl);
                rs.iter().map(|&r| psi_sq.evaluate_r(r)).collect()
            }
        };
        (rs, vals)
    }
}

fn basic_name(qn: Qn) -> String {
    format!("ψ<sub>{}{}{}</sub>", qn.n(), qn.l(), qn.m())
}

/// The minimum total probability enclosed within the bounding sphere of an atomic orbital.
pub const PROBABILITY_WITHIN_BOUND: f32 = 0.998;

fn bound(qn: Qn) -> f32 {
    const STEP: f32 = 0.05;

    let mut r = 0_f32;
    let mut probability = 0_f32;
    let psi_sq = RadialProbabilityDistribution::new(qn.into());

    while probability < PROBABILITY_WITHIN_BOUND {
        numerics::integrate_rk4_step(|r| psi_sq.evaluate_r(r), &mut r, &mut probability, STEP);
    }
    r
}

/// Get the conventional subshell name (s, p, d, f, etc.) for common (i.e., small) values of `l`;
/// will otherwise return `None`.
pub fn subshell_name(l: u32) -> Option<&'static str> {
    match l {
        0 => Some("s"),
        1 => Some("p"),
        2 => Some("d"),
        3 => Some("f"),
        4 => Some("g"),
        5 => Some("h"),
        6 => Some("i"),
        7 => Some("k"), // Note that "j" is skipped!
        _ => None,
    }
}

/// See attached Mathematica notebooks for the computation of test values.
#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;

    use super::{Radial, RadialPlot, PROBABILITY_WITHIN_BOUND};
    use crate::geometry::Point;
    use crate::numerics::{self, Evaluate};
    use crate::orbital::quantum_numbers::{Nl, Qn};

    static TEST_POINTS: Lazy<Vec<Point>> = Lazy::new(|| {
        vec![
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
        ]
    });

    macro_rules! test {
        ($fn_name:ident, $target:ty, $target_params:expr, $expected:expr) => {
            #[test]
            fn $fn_name() {
                let evaluator = <$target>::new($target_params);
                let calculated: Vec<_> = TEST_POINTS
                    .iter()
                    .map(|pt| evaluator.evaluate(pt))
                    .collect();
                assert_iterable_relative_eq!($expected, &calculated, max_relative = 1E-4_f32);
            }
        };
    }

    test!(
        test_radial_1_0,
        Radial,
        Nl::new(1, 0).unwrap(),
        &[
            0.00663625, 0.0058303, 0.00134232, 0.00109754, 0.0856609, 0.00137691, 0.00097705,
            0.00013556, 0.00017174, 0.00053425,
        ]
    );
    test!(
        test_radial_2_1,
        Radial,
        Nl::new(2, 1).unwrap(),
        &[
            0.06712, 0.0643393, 0.0386382, 0.0359008, 0.133092, 0.0389966, 0.0343976, 0.0161317,
            0.0177099, 0.0274495,
        ]
    );
    test!(
        test_radial_3_0,
        Radial,
        Nl::new(3, 0).unwrap(),
        &[
            -0.0224952, -0.0202023, 0.00281192, 0.00536186, -0.0491676, 0.00247796, 0.00675915,
            0.0223802, 0.0212521, 0.0131223
        ]
    );
    test!(
        test_radial_5_3,
        Radial,
        Nl::new(5, 3).unwrap(),
        &[
            0.00865692, 0.00894101, 0.0117124, 0.0120124, 0.00286186, 0.0116729, 0.0121757,
            0.0137583, 0.013689, 0.0129009,
        ]
    );

    #[test]
    fn test_radial_probability_density_unity() {
        Qn::enumerate_up_to_n(17)
            .unwrap()
            .filter(|qn| qn.m() == 0) // The radial component depends only on n and l.
            .map(|qn| RadialPlot::ProbabilityDistribution.sample(qn, 1_000))
            .for_each(|(xs, ys)| {
                approx::assert_abs_diff_eq!(
                    PROBABILITY_WITHIN_BOUND,
                    numerics::integrate_trapezoidal(&xs, &ys),
                    epsilon = 5E-4
                );
            });
    }
}
