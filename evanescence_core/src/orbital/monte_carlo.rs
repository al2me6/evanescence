//! An implementation of a Monte Carlo simulation to produce point cloud visualizations of orbitals.

use nanorand::{Rng, WyRand};

use super::hybrid::Hybrid;
use super::{Complex, Orbital, Qn, Real};
use crate::geometry::{ComponentForm, Point, PointValue};

/// Perform a Monte Carlo simulation of an orbital, generating a collection of points whose
/// distribution corresponds to one obtained by repeatedly measuring an electron in that orbital.
///
/// The algorithm operates as follows:
///
/// 1. Determine the maximum probability density attained by the wavefunction.
/// 2. Determine the radius of the ball sampled (see
///    [`EvaluateBounded::bound`](crate::numerics::EvaluateBounded::bound)).
/// 2. Repeat the following steps until the desired number of points is generated:
///     1. Generate a point randomly distributed within the ball of the aforementioned radius.
///     2. Generate a random number on \[0, 1\].
///     3. Evaluate the probability density of the wavefunction at the random point.
///     4. Taking the maximum density value to be 1, normalize the density value at the randomly
///        sampled point. This produces a value on \[0, 1\].
///     5. If the randomly generated number is greater than this normalized probability density,
///        keep the point. Otherwise, discard it.
///
/// It is computationally expensive to determine the maximum probability density of an orbital.
/// In this implementation, we instead approximate this maximum using another Monte Carlo
/// simulation by repeatedly sampling points and taking the maximum value obtained. This is
/// implemented in the [`estimate_maximum_value`](MonteCarlo::estimate_max_prob_density) function.
///
/// For the sake of performance, the random points sampled during the determination of the maximum
/// value are recycled for use in the main simulation.
///
/// For more information, see [Tully et al. 2013](https://doi.org/10.1021/ed300393s).
pub trait MonteCarlo: Orbital {
    /// The minimum number of points required to get a reasonable estimate of the maximum value
    /// attained by an orbital.
    fn minimum_estimation_samples(&self) -> usize {
        100_000
    }

    /// For a given [`Quality`], sample this many times more points for max value estimation.
    fn estimation_sample_factor(&self) -> usize {
        4
    }

    /// An optional factor that can be used to scale the computed max value as an approximation
    /// for the Monte Carlo simulation. This can significantly improve speed depending on the
    /// geometry of the specific orbital; see [`MonteCarlo::monte_carlo_simulate`].
    fn max_value_multiplier(&self) -> Option<f32> {
        None
    }

    /// Estimate the maximum probability density attained by an orbital by brute-force sampling.
    /// In addition to the maximum value, this function returns the points (and values) sampled
    /// for later use.
    fn estimate_max_prob_density(
        &self,
        num_samples: usize,
        rng: &mut WyRand,
    ) -> (f32, Vec<PointValue<Self::Output>>) {
        // Note that we force the origin to be sampled. This is to ensure that s orbitals are
        // accurately estimated: They attain their maximum probability density over a very small
        // area near the origin, which is difficult to hit when sampling randomly.
        let evaluated_points: Vec<_> = Point::sample_from_ball_with_origin_iter(self.bound(), rng)
            .map(|pt| self.evaluate_at(&pt))
            .take(num_samples)
            .collect();
        let max_prob_density = evaluated_points
            .iter()
            .map(|PointValue(_, val)| self.probability_density_of(*val))
            .reduce(f32::max)
            .expect("there should be at least one sample");
        (max_prob_density, evaluated_points)
    }

    /// Run a Monte Carlo simulation for the orbital of the given parameters at the requested point
    /// count, which also controls the number of points sampled for the maximum value estimation.
    ///
    /// Tne `use_fast_approximation` argument optionally enables an approximation: If enabled,
    /// the maximum probability density computed will be reduced by an (optional)
    /// implementation-supplied amount (specified in [`MonteCarlo::max_value_multiplier`]). This
    /// alleviates poor performance for orbitals where the probability density attains its maximum
    /// in a very small region of space, in which case the vast majority of space becomes
    /// excessively difficult to sample. For example, if the optimization is disabled, it takes
    /// approximately 200 000 000 samples to produce 50 000 points for the 5s orbital.
    ///
    /// Note that while slower, higher qualities may be required to obtain sufficiently detailed
    /// results for larger, more intricate orbitals. However, excessive quality for small orbitals
    /// may obstruct details while significantly degrading user experience.
    fn monte_carlo_simulate(
        &self,
        count: usize,
        use_fast_approximation: bool,
    ) -> ComponentForm<Self::Output> {
        let num_estimation_samples = usize::max(
            count * self.estimation_sample_factor(),
            self.minimum_estimation_samples(),
        );

        let mut point_rng = WyRand::new();
        let mut value_rng = WyRand::new();

        let (mut max_value, estimation_samples) =
            self.estimate_max_prob_density(num_estimation_samples, &mut point_rng);
        if use_fast_approximation {
            if let Some(multiplier) = self.max_value_multiplier() {
                max_value *= multiplier;
            }
        }
        let max_value = max_value;

        estimation_samples
            .into_iter() // Reuse the points sampled during estimation...
            .chain(
                // ...before generating new ones.
                Point::sample_from_ball_iter(self.bound(), &mut point_rng)
                    .map(|pt| self.evaluate_at(&pt)),
            )
            .filter(|PointValue(_, val)| {
                self.probability_density_of(*val) / max_value > value_rng.generate()
            })
            .take(count)
            .collect()
    }
}

fn atomic_orbital_mult(qn: Qn) -> f32 {
    1.0 / (0.05 * (Real::num_radial_nodes(qn) as f32).powi(3) + 1.0)
}

impl MonteCarlo for Real {
    fn max_value_multiplier(&self) -> Option<f32> {
        Some(atomic_orbital_mult(self.qn))
    }
}

impl MonteCarlo for Complex {
    fn max_value_multiplier(&self) -> Option<f32> {
        Some(atomic_orbital_mult(self.qn))
    }
}

impl MonteCarlo for Hybrid {}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::BufWriter;
    use std::path::PathBuf;

    use itertools::Itertools;
    use rayon::prelude::*;

    use super::MonteCarlo;
    use crate::numerics::integrators::integrate_simpson;
    use crate::numerics::statistics::kolmogorov_smirnov_test;
    use crate::orbital::atomic::RadialProbabilityDistribution;
    use crate::orbital::{Qn, Real};

    #[test]
    fn real_monte_carlo_radial_distribution() {
        #[derive(serde::Serialize)]
        struct Output {
            name: String,
            ks: f32,
            p: f32,
            cdf: Vec<f32>,
            rho: Vec<f32>,
        }

        const SAMPLES: usize = 10_000;

        // Circumvent garbled output due to threads writing concurrently.
        #[allow(clippy::format_in_format_args)]
        Qn::enumerate_up_to_n(5)
            .unwrap()
            .par_bridge()
            .map(|qn| {
                let orbital = Real::new(qn);
                let radial_probability_distribution = RadialProbabilityDistribution::new(qn.into());

                let (xs, ys, zs, _) = orbital
                    .monte_carlo_simulate(SAMPLES, true)
                    .into_components();
                let rho = {
                    // TODO: Refactor MonteCarlo to remove reconstitution tap-dance.
                    let mut rho = itertools::izip!(&xs, &ys, &zs)
                        .map(|(x, y, z)| (x * x + y * y + z * z).sqrt())
                        .collect_vec();
                    rho.sort_by(f32::total_cmp);
                    rho
                };

                let cdf = |r| {
                    integrate_simpson(
                        |s| radial_probability_distribution.evaluate_r(s),
                        0.,
                        r,
                        r / 40.,
                    )
                };
                let (ks_statistic, p) = kolmogorov_smirnov_test(&rho, cdf);

                println!("{}", format!("{qn} \tks = {ks_statistic} \tp = {p}"));

                if ks_statistic > 0.015 || p < 0.05 {
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
                            ks: ks_statistic,
                            p,
                            cdf: rho.iter().copied().map(cdf).collect(),
                            rho,
                        },
                    )
                    .unwrap();

                    eprintln!(
                        "{}",
                        format!(
                            "{qn}: K-S test failed; data exported to {}.",
                            out_path.to_string_lossy()
                        )
                    );
                    return Err(());
                }

                Ok(())
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
    }
}
