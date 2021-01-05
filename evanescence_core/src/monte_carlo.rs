use strum::{Display, EnumString};

use crate::numerics::Evaluation;
use crate::orbital::{self, Orbital};
use crate::utils::new_rng;
use crate::{geometry::Point, numerics::ComponentForm};

/// A set of predefined qualities (i.e., number of points computed) for
/// [`MonteCarlo::monte_carlo_simulate`] simulations.
///
/// These values have been empirically observed to produce reasonable results.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Display, EnumString)]
pub enum Quality {
    /// Produces "recognizable" results, but not much more.
    Minimum = 5_000,
    Low = 10_000,
    Medium = 25_000,
    /// Generally a good middle ground between performance and quality.
    High = 50_000,
    VeryHigh = 100_000,
    /// Should likely be avoided for all but the largest orbitals.
    Extreme = 250_000,
}

/// Perform a Monte Carlo simulation of an orbital, generating a collection of points
/// whose distribution corresponds to the geometry of said orbital.
pub trait MonteCarlo: Orbital {
    /// The minimum number of points required to get a reasonable estimate of
    /// the maximum value attained by an orbital.
    const MINIMUM_ESTIMATION_SAMPLES: usize = 50_000;

    /// Process values such that the results can be used to compute a maximum.
    fn value_comparator(value: Self::Output) -> f64;

    /// Estimate the maximum value attained by an orbital by brute-force sampling,
    /// using [`MonteCarlo::value_comparator`] as the metric by which values are compared.
    /// In addition to the maximum value, this function returns the points (and values)
    /// sampled for later use.
    fn estimate_maximum_value(
        params: Self::Parameters,
        num_samples: usize,
    ) -> (f64, Vec<Evaluation<Self::Output>>) {
        let evaluated_points: Vec<_> =
            Point::sample_from_ball_with_origin_iter(Self::estimate_radius(params))
                .map(|pt| Self::evaluate_at(params, &pt))
                .take(num_samples)
                .collect();
        let max_value = evaluated_points
            .iter()
            .map(|(_, val)| Self::value_comparator(*val))
            .fold(0.0, f64::max);
        (max_value, evaluated_points)
    }

    /// Run a Monte Carlo simulation for the orbital of the given parameters, at the
    /// requested quality, where quality corresponds to both the total number of points
    /// generated and the number of points sampled in the maximum value estimation.
    ///
    /// Note that while slower, higher qualities may be required to ortain sufficiently
    /// detailed results for larger, more intricate orbitals. However, excessive quality
    /// for small orbitals may obstruct details while significantly degrading user experience.
    fn monte_carlo_simulate(
        params: Self::Parameters,
        quality: Quality,
    ) -> ComponentForm<Self::Output> {
        let num_estimation_samples = (quality as usize * 2).max(Self::MINIMUM_ESTIMATION_SAMPLES);
        let mut rng = new_rng();
        let (max_value, estimation_samples) =
            Self::estimate_maximum_value(params, num_estimation_samples);
        estimation_samples
            .into_iter()
            .chain(
                Point::sample_from_ball_iter(Self::estimate_radius(params))
                    .map(|pt| Self::evaluate_at(params, &pt)),
            )
            .filter(|(_, val)| Self::value_comparator(*val) / max_value > rng.rand_float())
            .take(quality as usize)
            .collect::<Vec<_>>()
            .into()
    }
}

impl MonteCarlo for orbital::Real {
    #[inline]
    fn value_comparator(val: f64) -> f64 {
        val.abs()
    }
}
