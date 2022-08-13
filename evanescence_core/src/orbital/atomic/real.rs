use std::f32::consts::{FRAC_PI_2, PI};

use na::Point1;

use super::Radial;
use crate::geometry::point::{SphericalPoint3, SphericalPoint3Ext};
use crate::geometry::region::{BallCenteredAtOrigin, BoundingRegion};
use crate::numerics;
use crate::numerics::monte_carlo::accept_reject::AcceptRejectParameters;
use crate::numerics::special::orthogonal_polynomials::renormalized_associated_legendre;
use crate::numerics::spherical_harmonics::RealSphericalHarmonic;
use crate::numerics::statistics::Distribution;
use crate::numerics::Function;
use crate::orbital::quantum_numbers::{Lm, Qn};
use crate::orbital::Orbital;

/// Implementation of the real hydrogenic orbitals.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Real {
    qn: Qn,
    radial: Radial,
    sph: RealSphericalHarmonic,
}

impl Real {
    pub fn new(qn: Qn) -> Self {
        Self {
            qn,
            radial: Radial::new(qn.into()),
            sph: RealSphericalHarmonic::new(qn.into()),
        }
    }

    pub fn name_qn(qn: Qn) -> String {
        if let (Some(subshell), Some(linear_combination)) = (
            super::subshell_name(qn.l()),
            RealSphericalHarmonic::expression(qn.into()),
        ) {
            format!("{}{subshell}<sub>{linear_combination}</sub>", qn.n())
        } else {
            super::basic_name(qn)
        }
    }
}

impl Function<3, SphericalPoint3> for Real {
    type Output = f32;

    #[inline]
    fn evaluate(&self, point: &SphericalPoint3) -> f32 {
        self.radial.evaluate(&[point.r()].into()) * self.sph.evaluate(point)
    }
}

impl BoundingRegion<3, SphericalPoint3> for Real {
    type Geometry = BallCenteredAtOrigin;

    fn bounding_region(&self) -> Self::Geometry {
        BallCenteredAtOrigin {
            radius: super::bound(self.qn),
        }
    }
}

impl Distribution<3, SphericalPoint3> for Real {
    #[inline]
    fn probability_density_of(&self, value: Self::Output) -> f32 {
        value * value
    }
}

impl Orbital<SphericalPoint3> for Real {
    /// Try to give the orbital's conventional name (ex. `4d_{z^2}`) before falling back to giving
    /// the quantum numbers only (ex. `ψ_{420}`).
    fn name(&self) -> String {
        Self::name_qn(self.qn)
    }
}

impl AcceptRejectParameters<3, SphericalPoint3> for Real {
    // TODO: custom maximum impl.
    fn accept_threshold_fudge(&self) -> Option<f32> {
        Some(super::accept_threshold_modifier(self.qn))
    }
}

impl Real {
    /// Give the number of radial nodes in an orbital.
    pub fn num_radial_nodes(qn: Qn) -> u32 {
        qn.n() - qn.l() - 1
    }

    /// Give the number of angular nodes in an orbital.
    pub fn num_angular_nodes(qn: Qn) -> u32 {
        qn.l()
    }

    /// Give the number of conical angular nodes in an orbital.
    pub fn num_conical_nodes(lm: Lm) -> u32 {
        lm.l() - lm.m().unsigned_abs()
    }

    /// Give the number of planar angular nodes in an orbital.
    pub fn num_planar_nodes(lm: Lm) -> u32 {
        lm.m().unsigned_abs()
    }

    /// Give the `r` values of all radial nodes of a given `n` and `l` pair.
    #[allow(clippy::missing_panics_doc)] // The `assert_eq` is an internal sanity check.
    pub fn radial_node_positions(qn: Qn) -> Vec<f32> {
        let radial = Radial::new(qn.into());
        let roots = numerics::root_finding::find_roots_in_interval_brent(
            0.05..=super::bound(qn),
            100,
            |r| radial.evaluate(&Point1::new(r)),
        )
        .expect("root finder did not converge");
        assert_eq!(
            roots.len(),
            Self::num_radial_nodes(qn) as usize,
            "not all radial nodes were found"
        );
        roots
    }

    /// Give the theta angles of all conical nodes of a given `l` and `m` pair.
    #[allow(clippy::missing_panics_doc)] // The `assert_eq` is an internal sanity check.
    pub fn conical_node_angles(lm: Lm) -> Vec<f32> {
        let roots = numerics::root_finding::find_roots_in_interval_brent(0.0..=PI, 90, |theta| {
            renormalized_associated_legendre((lm.l(), lm.m().unsigned_abs()), theta.cos())
        })
        .expect("root finder did not converge");
        assert_eq!(
            roots.len(),
            Self::num_conical_nodes(lm) as usize,
            "not all conical node angles were found"
        );
        roots
    }

    /// Give the phi angles of all planar nodes of a given `l` and `m` pair.
    pub fn planar_node_angles(lm: Lm) -> Vec<f32> {
        // Construct roots of the phi part of the corresponding real spherical harmonic:
        // `cos(m * phi)`    if m > 0
        // 1                 if m = 0
        // `sin(|m| * phi)`  if m < 0
        let m = lm.m();
        let m_abs = m.unsigned_abs();
        if m == 0 {
            return vec![];
        }
        #[allow(clippy::maybe_infinite_iter)] // Arithmetic sequence; will terminate.
        (0..)
            .map(|n| n as f32 * PI)
            .map(|root| if m > 0 { root + FRAC_PI_2 } else { root })
            .map(|root| root / m_abs as f32)
            .take_while(|&root| root < PI)
            .collect()
    }
}

/// See attached Mathematica notebooks for the computation of test values.
#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::BufWriter;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use itertools::Itertools;
    use na::{vector, Point1};
    use rayon::prelude::*;

    use super::Real;
    use crate::geometry::point::{SphericalPoint3, SphericalPoint3Ext};
    use crate::geometry::region::BoundingRegion;
    use crate::geometry::storage::PointValue;
    use crate::numerics::integrators::integrate_simpson;
    use crate::numerics::monte_carlo::accept_reject::AcceptReject;
    use crate::numerics::monte_carlo::MonteCarlo;
    use crate::numerics::statistics::kolmogorov_smirnov::kolmogorov_smirnov_test;
    use crate::numerics::statistics::ProbabilityDensityEvaluator;
    use crate::numerics::Function;
    use crate::orbital::atomic::{RadialProbabilityDistribution, PROBABILITY_WITHIN_BOUND};
    use crate::orbital::quantum_numbers::{Lm, Qn};

    #[test]
    fn real_radial_nodes() {
        Qn::enumerate_up_to_n(10)
            .unwrap()
            .filter(|qn| qn.m() == 0)
            .map(|qn| (qn, Real::radial_node_positions(qn)))
            .for_each(|(qn, pts)| {
                println!("[{qn}]: {}", pts.iter().join(", "));
            });
    }

    #[test]
    fn real_conical_node_angles() {
        Qn::enumerate_up_to_n(10)
            .unwrap()
            .map(Lm::from)
            .unique()
            .map(|lm| (lm, Real::conical_node_angles(lm)))
            .for_each(|(lm, pts)| {
                println!(
                    "[{lm:?}]: {}",
                    pts.iter().map(|theta| theta.to_degrees()).join(", ")
                );
            });
    }

    #[test]
    fn real_planar_node_angles() {
        Qn::enumerate_up_to_n(10)
            .unwrap()
            .map(Lm::from)
            .unique()
            .map(|lm| (lm, Real::planar_node_angles(lm)))
            .for_each(|(lm, pts)| {
                assert_eq!(pts.len(), Real::num_planar_nodes(lm) as usize);
                println!(
                    "[{lm:?}]: {}",
                    pts.iter().map(|phi| phi.to_degrees()).join(", ")
                );
            });
    }

    #[test]
    fn real_probability_unity() {
        let qns = Qn::enumerate_up_to_n(7).unwrap().step_by(3).collect_vec();
        qns.into_par_iter().for_each(|qn| {
            let psi_sq = ProbabilityDensityEvaluator::new(Real::new(qn));
            let bound = psi_sq.bounding_region().radius;
            let prob = integrate_simpson_multiple!(
                psi_sq.evaluate(&SphericalPoint3::from(vector![x, y, z])),
                step: bound / 40_f32,
                x: (-bound, bound),
                y: (-bound, bound),
                z: (-bound, bound),
            );
            println!("{qn}: {prob}");
            assert!(PROBABILITY_WITHIN_BOUND < prob && prob < 1_f32 + 5E-5);
        });
    }

    #[test]
    fn real_monte_carlo_radial_distribution() {
        #[derive(serde::Serialize)]
        struct Output {
            name: String,
            ks: f32,
            p: f32,
            cdf: Vec<f32>,
            rhos: Vec<Vec<f32>>,
        }

        const SAMPLES: usize = 10_000;
        const TRIALS: usize = 6;
        const KS_THRESHOLD: f32 = 0.02;
        const P_THRESHOLD: f32 = 0.05;
        const MAX_FAILS: usize = 2;

        let failed_count = AtomicUsize::new(0);

        // Circumvent garbled output due to threads writing concurrently.
        #[allow(clippy::format_in_format_args)]
        Qn::enumerate_up_to_n(5)
            .unwrap()
            .par_bridge()
            .for_each(|qn| {
                let radial_probability_distribution = RadialProbabilityDistribution::new(qn.into());
                let cdf = |r| {
                    integrate_simpson(
                        |s| radial_probability_distribution.evaluate(&Point1::new(s)),
                        0.,
                        r,
                        (r / 40.).max(0.005),
                    )
                };

                let (rhos, (ks_statistics, ps)): (Vec<_>, (Vec<_>, Vec<_>)) =
                    std::iter::repeat_with(|| {
                        let mut sampler = AcceptReject::new(Real::new(qn));
                        let rho = sampler
                            .simulate(SAMPLES)
                            .into_iter()
                            .map(|PointValue(pt, _)| pt.r())
                            .sorted_by(f32::total_cmp)
                            .collect_vec();

                        let ks_result = kolmogorov_smirnov_test(&rho, cdf);
                        (rho, ks_result)
                    })
                    .take(TRIALS)
                    .unzip();

                print!(
                    "{}",
                    format!("{qn}\n\tks = {ks_statistics:?}\n\tp  = {ps:?}\n")
                );

                let ks_failed_count = ks_statistics
                    .iter()
                    .filter(|&&ks| ks > KS_THRESHOLD)
                    .count();
                let p_failed_count = ps.iter().filter(|&&p| p < P_THRESHOLD).count();

                if ks_failed_count > MAX_FAILS || p_failed_count > MAX_FAILS {
                    let mut out_path: PathBuf =
                        [env!("CARGO_MANIFEST_DIR"), "test_output"].iter().collect();
                    fs::create_dir_all(&out_path).unwrap();
                    out_path.push(format!(
                        "{}_test-real-monte-carlo_{}{}{}.json",
                        chrono::offset::Utc::now()
                            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                            .replace(':', ""),
                        qn.n(),
                        qn.l(),
                        qn.m().to_string().replace('-', "n"),
                    ));
                    let out_file = File::create(&out_path).unwrap();

                    serde_json::to_writer(
                        BufWriter::new(out_file),
                        &Output {
                            name: qn.to_string(),
                            ks: *ks_statistics.iter().max_by(|a, b| a.total_cmp(b)).unwrap(),
                            p: *ps.iter().min_by(|a, b| a.total_cmp(b)).unwrap(),
                            cdf: rhos[0].iter().copied().map(cdf).collect(),
                            rhos,
                        },
                    )
                    .unwrap();

                    eprint!(
                        "{}",
                        format!(
                            "{qn}: K-S test failed; data exported to {}.\n",
                            out_path.to_string_lossy()
                        )
                    );
                    failed_count.fetch_add(1, Ordering::Relaxed);
                }
            });

        let failed_count = failed_count.into_inner();
        if failed_count > 0 {
            panic!("{failed_count} K-S tests failed!")
        }
    }
}
