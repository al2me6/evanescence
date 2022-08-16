//! This module implements real and complex atomic orbitals, including their radial and
//! angular components. Also included are probability and probability distribution functions
//! for the radial wavefunction.

pub mod complex;
pub mod real;

use na::Point1;

use crate::geometry::point::IPoint;
use crate::numerics;
use crate::numerics::polynomial::Polynomial;
use crate::numerics::special::orthogonal_polynomials;
use crate::numerics::Function;
use crate::orbital::quantum_numbers::{Nl, Qn};

/// Implementation of the radial component of the hydrogenic wavefunction.
#[derive(Clone, Debug)]
pub struct Radial {
    nl: Nl,
    normalization: f32,
    associated_laguerre: Polynomial,
}

impl PartialEq for Radial {
    fn eq(&self, other: &Self) -> bool {
        self.nl == other.nl
    }
}

impl Eq for Radial {}

impl Radial {
    pub fn new(nl: Nl) -> Self {
        let (n, l) = (nl.n(), nl.l());
        Self {
            nl,
            normalization: Self::normalization_factor(n, l),
            associated_laguerre: orthogonal_polynomials::associated_laguerre(n - l - 1, 2 * l + 1),
        }
    }

    /// Calculate the radial wavefunction normalization factor,
    /// `√( (2Z/n)^3 * (n-l-1)! / (2n * (n+l)!) )`.
    fn normalization_factor(n: u32, l: u32) -> f32 {
        // (n-l-1)! / (n+l)! = 1 / [(n-l) * (n-l+1) * ... * (n+l-1) * (n+l)].
        let factorial_factor = ((n - l)..=(n + l)).map(|k| k as f32).product::<f32>();
        // Where we've taken `(2Z/n)^3 / 2n` out ouf the square root.
        2.0 / factorial_factor.sqrt() / ((n * n) as f32)
    }
}

impl Function<1> for Radial {
    type Output = f32;

    #[inline]
    fn evaluate(&self, point: &Point1<f32>) -> Self::Output {
        let r = point.coordinates().x;
        let rho = 2.0 * r / (self.nl.n() as f32);
        self.normalization
            * (-rho / 2.0).exp()
            * rho.powi(self.nl.l() as i32)
            * self.associated_laguerre.evaluate_horner(rho)
    }
}

/// The radial probability distribution, `r^2R^2`.
pub struct RadialProbabilityDistribution(Radial);

impl RadialProbabilityDistribution {
    pub fn new(nl: Nl) -> Self {
        Self(Radial::new(nl))
    }
}

impl Function<1> for RadialProbabilityDistribution {
    type Output = f32;

    #[inline]
    fn evaluate(&self, point: &Point1<f32>) -> Self::Output {
        let r = point.coords.x;
        #[allow(non_snake_case)] // Mathematical convention.
        let R = self.0.evaluate(point);
        r * r * R * R
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
        numerics::integrators::integrate_simpson_step(
            |r| psi_sq.evaluate(&Point1::new(r)),
            &mut r,
            &mut probability,
            STEP,
        );
    }
    r
}

fn accept_threshold_modifier(qn: Qn) -> f32 {
    1.0 / (0.05 * (real::Real::num_radial_nodes(qn) as f32).powi(3) + 1.0)
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
    use na::{vector, Point1};

    use super::{Radial, PROBABILITY_WITHIN_BOUND};
    use crate::geometry::storage::struct_of_arrays::ToSoa;
    use crate::numerics;
    use crate::numerics::Function;
    use crate::orbital::atomic::RadialProbabilityDistribution;
    use crate::orbital::quantum_numbers::{Nl, Qn};

    #[test]
    fn radial() {
        #[derive(serde::Deserialize)]
        struct Sample {
            pt: f64,
            val: f64,
        }

        #[derive(serde::Deserialize)]
        struct TestCase {
            n: u32,
            l: u32,
            samples: Vec<Sample>,
        }

        let json = std::fs::read_to_string(
            [env!("CARGO_MANIFEST_DIR"), "mathematica", "radial.json"]
                .iter()
                .collect::<std::path::PathBuf>(),
        )
        .unwrap();
        let data: Vec<TestCase> = serde_json::from_str(&json).unwrap();

        for TestCase { n, l, samples } in data {
            let radial = Radial::new(Nl::new(n, l).unwrap());

            #[allow(clippy::cast_possible_truncation)]
            for Sample { pt, val: expected } in samples {
                let expected = expected as f32;
                let computed = radial.evaluate(&Point1::new(pt as f32));
                let tolerance = if n < 9 { 1E-7 } else { 2E-2 };

                assert!(
                    approx::relative_eq!(expected, computed, max_relative = tolerance),
                    "R_{n}^{l}({pt}):\n\texpected = {expected}\n\t     got = {computed}\n"
                );
            }
        }
    }

    #[test]
    fn radial_probability_unity() {
        // The radial component depends only on n and l.
        for qn in Qn::enumerate_up_to_n(15).unwrap().filter(|qn| qn.m() == 0) {
            let ([xs], ys) = RadialProbabilityDistribution::new(qn.into())
                .sample_from_line_segment(vector![0.]..=vector![super::bound(qn)], 1_000)
                .to_soa_components();
            approx::assert_abs_diff_eq!(
                PROBABILITY_WITHIN_BOUND,
                numerics::integrators::integrate_trapezoidal(&xs, &ys),
                epsilon = 0.000_75,
            );
        }
    }
}
