use getset::Getters;

use crate::geometry::Point;
use crate::numerics::new_rng;
use crate::orbital::{QuantumNumbers, RealOrbital, Wavefunction};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Minimum = 5_000,
    Low = 10_000,
    Medium = 25_000,
    High = 50_000,
    VeryHigh = 100_000,
    Extreme = 250_000,
}

pub type EvaluationResult<T> = (Point, T);

#[derive(Getters)]
#[getset(get = "pub")]
pub struct SimulationResult<T> {
    xs: Vec<f64>,
    ys: Vec<f64>,
    zs: Vec<f64>,
    vals: Vec<T>,
}

impl<T> SimulationResult<T> {
    pub fn into_components(self) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<T>) {
        (self.xs, self.ys, self.zs, self.vals)
    }
}

impl<T> From<Vec<EvaluationResult<T>>> for SimulationResult<T> {
    fn from(v: Vec<EvaluationResult<T>>) -> Self {
        let len = v.len();
        let (mut xs, mut ys, mut zs, mut vals) = (
            Vec::with_capacity(len),
            Vec::with_capacity(len),
            Vec::with_capacity(len),
            Vec::with_capacity(len),
        );
        v.into_iter().for_each(|(pt, val)| {
            xs.push(pt.x());
            ys.push(pt.y());
            zs.push(pt.z());
            vals.push(val);
        });
        SimulationResult { xs, ys, zs, vals }
    }
}

pub trait MonteCarlo: Wavefunction {
    const MINIMUM_ESTIMATION_SAMPLES: usize = 50_000;

    fn estimate_radius(params: Self::Parameters) -> f64;

    fn value_comparator(value: Self::Output) -> f64;

    fn estimate_maximum_value(
        params: Self::Parameters,
        num_samples: usize,
    ) -> (f64, Vec<EvaluationResult<Self::Output>>) {
        let evaluated_points: Vec<_> =
            Point::sample_from_ball_with_origin_iter(Self::estimate_radius(params))
                .map(|pt| (pt, Self::evaluate(params, &pt)))
                .take(num_samples)
                .collect();
        let max_value = evaluated_points
            .iter()
            .map(|(_, val)| Self::value_comparator(*val))
            .fold(0.0, f64::max);
        (max_value, evaluated_points)
    }

    fn monte_carlo_simulate(
        params: Self::Parameters,
        quality: Quality,
    ) -> SimulationResult<Self::Output> {
        let num_estimation_samples = (quality as usize * 2).max(Self::MINIMUM_ESTIMATION_SAMPLES);
        let mut rng = new_rng();
        let (max_value, estimation_samples) =
            Self::estimate_maximum_value(params, num_estimation_samples);
        estimation_samples
            .into_iter()
            .chain(
                Point::sample_from_ball_iter(Self::estimate_radius(params))
                    .map(|pt| (pt, Self::evaluate(params, &pt))),
            )
            .filter(|(_, val)| Self::value_comparator(*val) / max_value > rng.rand_float())
            .take(quality as usize)
            .collect::<Vec<_>>() // Seems to increase performance by ~10%.
            .into()
    }
}

impl MonteCarlo for RealOrbital {
    /// An empirically derived heuristic for estimating the maximum radius of
    /// an orbital. See the attached Mathematica notebook `radial_wavefunction.nb`
    /// for plots.
    #[inline]
    fn estimate_radius(qn: QuantumNumbers) -> f64 {
        let n = qn.n() as f64;
        n * (2.5 * n - 0.625 * qn.l() as f64 + 3.0)
    }

    #[inline]
    fn value_comparator(val: f64) -> f64 {
        val.abs()
    }
}
