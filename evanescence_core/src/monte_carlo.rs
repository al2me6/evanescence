//! An implementation of a Monte Carlo simulation to produce point cloud visualizations of orbitals.

use nanorand::tls_rng;
use strum::{Display, EnumString};

use crate::geometry::{ComponentForm, Point, PointValue};
use crate::orbital::{self, Orbital, QuantumNumbers};
use crate::rand_f32;

/// A set of predefined qualities (i.e., number of points computed) for sampling orbitals, either
/// for Monte Carlo simulations or plotting.
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

impl Quality {
    /// Get the number of points that should be sampled for a line plot.
    #[allow(clippy::integer_division)] // Intentional.
    #[inline]
    pub fn for_line(self) -> usize {
        self as usize / 100
    }

    /// Get the number of points that should be sampled for a plane/cross-section plot.
    #[allow(
        clippy::cast_possible_truncation, // Unlikely consider the size of the values we work with.
        clippy::integer_division, // Intentional.
        clippy::cast_sign_loss, // A square root cannot be negative.
    )]
    #[inline]
    pub fn for_grid(self) -> usize {
        (self as usize as f32).sqrt() as usize / 2
    }
}

/// Perform a Monte Carlo simulation of an orbital, generating a collection of points whose
/// distribution corresponds to the geometry of said orbital.
///
/// The algorithm operates as follows:
///
/// 1. Determine the maximum absolute value attained by the wavefunction.
/// 2. Determine the radius of the ball sampled (see [`Orbital::estimate_radius`]).
/// 2. Repeat the following steps until the desired number of points is generated:
///     * Generate a point randomly distributed within the ball of the aforementioned radius.
///     * Generate a random number on \[0, 1\].
///     * Evaluate the wavefunction at the random point.
///     * Taking the maximum wavefunction value to be 1, normalize the absolute value of the
///       value computed at the random value at the point. This produces a value on \[0, 1\].
///     * If the randomly generated number is greater than this normalized wavefunction value,
///       keep the point. Otherwise, discard it.
///
/// It is computationally expensive to determine the maximum absolute value of an orbital. In this
/// implementation, we instead approximate this maximum using another Monte Carlo simulation by
/// repeatedly sampling points and taking the maximum value obtained. This is implemented in the
/// [`estimate_maximum_value`](MonteCarlo::estimate_maximum_value) function.
///
/// For the sake of performance, the random points sampled during the determination of the maximum
/// value are recycled for use in the main simulation.
///
/// For more information, see [Tully et al. 2013](https://doi.org/10.1021/ed300393s).
pub trait MonteCarlo: Orbital {
    /// The minimum number of points required to get a reasonable estimate of the maximum value
    /// attained by an orbital.
    const MINIMUM_ESTIMATION_SAMPLES: usize = 50_000;

    /// Process values such that the results can be used to compute a maximum.
    fn value_comparator(value: Self::Output) -> f32;

    /// Estimate the maximum value attained by an orbital by brute-force sampling, using
    /// [`MonteCarlo::value_comparator`] as the metric by which values are compared. In addition
    /// to the maximum value, this function returns the points (and values) sampled for later use.
    fn estimate_maximum_value(
        qn: QuantumNumbers,
        num_samples: usize,
    ) -> (f32, Vec<PointValue<Self::Output>>) {
        // Note that we force the origin to be sampled. This is to ensure that s orbitals are
        // accurately estimated: They attain their maximum value over a very small area near the
        // origin, which is difficult to hit when sampling randomly.
        let evaluated_points: Vec<_> =
            Point::sample_from_ball_with_origin_iter(Self::estimate_radius(qn))
                .map(|pt| Self::evaluate_at(qn, &pt))
                .take(num_samples)
                .collect();
        let max_value = evaluated_points
            .iter()
            .map(|(_, val)| Self::value_comparator(*val))
            .fold(0.0, f32::max);
        (max_value, evaluated_points)
    }

    /// Run a Monte Carlo simulation for the orbital of the given parameters, at the requested
    /// quality, where quality corresponds to both the total number of points generated and the
    /// number of points sampled in the maximum value estimation.
    ///
    /// Note that while slower, higher qualities may be required to ortain sufficiently detailed
    /// results for larger, more intricate orbitals. However, excessive quality for small orbitals
    /// may obstruct details while significantly degrading user experience.
    fn monte_carlo_simulate(qn: QuantumNumbers, quality: Quality) -> ComponentForm<Self::Output> {
        let num_estimation_samples = (quality as usize * 2).max(Self::MINIMUM_ESTIMATION_SAMPLES);
        let mut rng = tls_rng();
        let (max_value, estimation_samples) =
            Self::estimate_maximum_value(qn, num_estimation_samples);
        estimation_samples
            .into_iter()
            .chain(
                Point::sample_from_ball_iter(Self::estimate_radius(qn))
                    .map(|pt| Self::evaluate_at(qn, &pt)),
            )
            .filter(|(_, val)| Self::value_comparator(*val) / max_value > rand_f32!(rng))
            .take(quality as usize)
            .collect::<Vec<_>>()
            .into()
    }
}

impl MonteCarlo for orbital::Real {
    #[inline]
    fn value_comparator(val: f32) -> f32 {
        val.abs()
    }
}
