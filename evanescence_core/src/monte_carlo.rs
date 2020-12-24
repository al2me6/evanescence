use crate::{geometry::Point, numerics::new_rng};

use crate::orbital::{QuantumNumbers, RealOrbital, Wavefunction};

pub trait MonteCarlo {
    type Output;
    const MINIMUM_ESTIMATION_SAMPLES: usize = 50_000;

    fn estimate_radius(qn: QuantumNumbers) -> f64;
    fn estimate_maximum_value(
        qn: QuantumNumbers,
        num_samples: usize,
    ) -> (f64, Vec<(Point, Self::Output)>);
    fn monte_carlo_simulate(qn: QuantumNumbers, quality: Quality) -> Vec<(Point, Self::Output)>;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Minimum = 5_000,
    Low = 10_000,
    Medium = 25_000,
    High = 50_000,
    VeryHigh = 100_000,
    Extreme = 250_000,
}

impl MonteCarlo for RealOrbital {
    type Output = f64;

    /// An empirically derived heuristic for estimating the maximum radius of
    /// an orbital. See the attached Mathematica notebook `radial_wavefunction.nb`
    /// for plots.
    #[inline]
    fn estimate_radius(qn: QuantumNumbers) -> f64 {
        let n = qn.n() as f64;
        n * (2.5 * n - 0.625 * qn.l() as f64 + 3.0)
    }

    fn estimate_maximum_value(
        qn: QuantumNumbers,
        num_samples: usize,
    ) -> (f64, Vec<(Point, Self::Output)>) {
        let evaluated_points: Vec<_> =
            Point::sample_from_ball_with_origin_iter(Self::estimate_radius(qn))
                .map(|pt| (pt, Self::evaluate(qn, &pt)))
                .take(num_samples)
                .collect();
        let max_value = evaluated_points
            .iter()
            .map(|(_, val)| val.abs())
            .fold_first(|a, b| if a > b { a } else { b })
            .expect("estimation requires at least one sample");
        (max_value, evaluated_points)
    }

    fn monte_carlo_simulate(qn: QuantumNumbers, quality: Quality) -> Vec<(Point, Self::Output)> {
        let num_estimation_samples = (quality as usize * 2).max(Self::MINIMUM_ESTIMATION_SAMPLES);
        let mut rng = new_rng();
        let (max_value, estimation_samples) =
            Self::estimate_maximum_value(qn, num_estimation_samples);
        estimation_samples
            .into_iter()
            .chain(
                Point::sample_from_ball_iter(Self::estimate_radius(qn))
                    .map(|pt| (pt, Self::evaluate(qn, &pt))),
            )
            .filter(|(_, value)| value.abs() / max_value > rng.rand_float())
            .take(quality as usize)
            .collect()
    }
}
