//! An implementation of a Monte Carlo simulation to produce point cloud visualizations of orbitals.

use nanorand::WyRand;
use strum::{Display, EnumIter, EnumString};

use crate::geometry::{ComponentForm, Point, PointValue};
use crate::orbital::{self, Orbital};
use crate::rand_f32;

/// A set of predefined qualities (i.e., number of points computed) for sampling orbitals, either
/// for Monte Carlo simulations or plotting.
///
/// These values have been empirically observed to produce reasonable results.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Display, EnumString, EnumIter)]
pub enum Quality {
    /// Produces "recognizable" results, but not much more.
    Minimum = 5_000,
    Low = 10_000,
    Medium = 25_000,
    /// Generally a good middle ground between performance and quality.
    High = 50_000,
    #[strum(serialize = "Very high")]
    VeryHigh = 100_000,
    /// Should likely be avoided for all but the largest orbitals.
    Extreme = 250_000,
}

impl Default for Quality {
    fn default() -> Self {
        Self::Low
    }
}

#[allow(
    clippy::cast_possible_truncation, // Discriminants are small enough.
    clippy::cast_sign_loss, // Roots of positive numbers are positive.
    clippy::integer_division, // Intentional.
)]
impl Quality {
    /// Get the number of points that should be sampled for a line plot.
    #[inline]
    pub fn for_line(self) -> usize {
        self as usize / 100
    }

    /// Get the number of points that should be sampled for a plane/cross-section plot.
    #[inline]
    pub fn for_grid(self) -> usize {
        ((self as usize as f32).sqrt() as usize / 2) | 0b1 // Force the number to be odd.
    }

    /// Get the number of points that should be sampled for an isosurface plot.
    #[inline]
    pub fn for_isosurface(self) -> usize {
        (self as usize as f32 * 2.0).cbrt() as usize | 0b1 // Force the number to be odd.
    }
}

/// Perform a Monte Carlo simulation of an orbital, generating a collection of points whose
/// distribution corresponds to one obtained by repeatedly measuring an electron in that orbital.
///
/// The algorithm operates as follows:
///
/// 1. Determine the maximum probability density attained by the wavefunction.
/// 2. Determine the radius of the ball sampled (see [`Orbital::estimate_radius`]).
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
    const MINIMUM_ESTIMATION_SAMPLES: usize = 50_000;

    /// An optional factor that can be used to scale the computed max value as an approximation
    /// for the Monte Carlo simulation. This can significantly improve speed depending on the
    /// geometry of the specific orbital; see [`MonteCarlo::monte_carlo_simulate`].
    fn max_value_multiplier(_params: &Self::Parameters) -> Option<f32> {
        None
    }

    /// Estimate the maximum probability density attained by an orbital by brute-force sampling.
    /// In addition to the maximum value, this function returns the points (and values) sampled
    /// for later use.
    fn estimate_max_prob_density(
        params: &Self::Parameters,
        num_samples: usize,
        rng: &mut WyRand,
    ) -> (f32, Vec<PointValue<Self::Output>>) {
        // Note that we force the origin to be sampled. This is to ensure that s orbitals are
        // accurately estimated: They attain their maximum probability density over a very small
        // area near the origin, which is difficult to hit when sampling randomly.
        let evaluated_points: Vec<_> =
            Point::sample_from_ball_with_origin_iter(Self::estimate_radius(params), rng)
                .map(|pt| Self::evaluate_at(params, &pt))
                .take(num_samples)
                .collect();
        let max_prob_density = evaluated_points
            .iter()
            .map(|PointValue(_, val)| Self::probability_density_of(*val))
            .reduce(f32::max)
            .expect("there should be at least one sample");
        (max_prob_density, evaluated_points)
    }

    /// Run a Monte Carlo simulation for the orbital of the given parameters, at the requested
    /// quality, where quality controls both the total number of points generated and the number
    /// of points sampled in the maximum value estimation.
    ///
    /// Tne `use_fast_approximation` argument optionally enables an approximation: If enabled,
    /// the maximum probability density computed will be reduced by an (optional)
    /// implementation-supplied amount (specified in [`MonteCarlo::max_value_multiplier`]). This
    /// alleviates poor performance for orbitals where the probability density attains its maximum
    /// in a very small region of space, in which case the vast majority of space becomes
    /// excessively difficult to sample. For example, if the optimization is disabled, it takes
    /// approximately 200 000 000 samples to produce 50 000 points for the 5s orbital.
    ///
    /// Note that while slower, higher qualities may be required to ortain sufficiently detailed
    /// results for larger, more intricate orbitals. However, excessive quality for small orbitals
    /// may obstruct details while significantly degrading user experience.
    fn monte_carlo_simulate(
        params: &Self::Parameters,
        quality: Quality,
        use_fast_approximation: bool,
    ) -> ComponentForm<Self::Output> {
        let num_estimation_samples = (quality as usize * 2).max(Self::MINIMUM_ESTIMATION_SAMPLES);

        let mut point_rng = WyRand::new();
        let mut value_rng = WyRand::new();

        let (mut max_value, estimation_samples) =
            Self::estimate_max_prob_density(params, num_estimation_samples, &mut point_rng);
        if use_fast_approximation {
            if let Some(multiplier) = Self::max_value_multiplier(params) {
                max_value *= multiplier;
            }
        }
        let max_value = max_value;

        estimation_samples
            .into_iter() // Reuse the points sampled during estimation...
            .chain(
                // ...before generating new ones.
                Point::sample_from_ball_iter(Self::estimate_radius(params), &mut point_rng)
                    .map(|pt| Self::evaluate_at(params, &pt)),
            )
            .filter(|PointValue(_, val)| {
                Self::probability_density_of(*val) / max_value > rand_f32!(value_rng)
            })
            .take(quality as usize)
            .collect::<Vec<_>>() // Faster than coverting to ComponentForm directly.
            .into()
    }
}

impl MonteCarlo for orbital::Real {
    fn max_value_multiplier(params: &Self::Parameters) -> Option<f32> {
        Some(1.0 / (0.05 * (Self::num_radial_nodes(params) as f32).powi(3) + 1.0))
    }
}

impl MonteCarlo for orbital::Complex {
    fn max_value_multiplier(params: &Self::Parameters) -> Option<f32> {
        orbital::Real::max_value_multiplier(params)
    }
}

impl MonteCarlo for orbital::Hybrid {}
