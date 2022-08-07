use std::iter;

use super::MonteCarlo;
use crate::geometry::region::{BoundingRegion, Region};
use crate::geometry::{Point, PointValue};
use crate::numerics::random::WyRand;
use crate::numerics::statistics::Distribution;
use crate::numerics::Evaluate;

pub trait MaximumInBoundingRegion: BoundingRegion + Distribution {
    fn maximum(&self) -> f32 {
        let mut rng = WyRand::new();
        let region = self.bounding_region();
        // TODO: samplea adaptively?
        Iterator::chain(
            iter::once(Point::ORIGIN_EPSILON),
            iter::repeat_with(|| region.sample(&mut rng)),
        )
        .take(200_000)
        .map(|pt| self.probability_density(&pt))
        .reduce(f32::max)
        .expect("there should be at least one sample")
    }
}

pub trait AcceptRejectFudge {
    fn accept_threshold_modifier(&self) -> Option<f32> {
        None
    }
}

pub struct AcceptReject<T> {
    distribution: T,
    maximum: f32,
}

impl<T> AcceptReject<T>
where
    T: Distribution + MaximumInBoundingRegion + AcceptRejectFudge,
{
    pub fn new(distribution: T) -> Self {
        let mut maximum = distribution.maximum();
        if let Some(modifier) = distribution.accept_threshold_modifier() {
            maximum *= modifier;
        }
        Self {
            distribution,
            maximum,
        }
    }
}

impl<T> MonteCarlo for AcceptReject<T>
where
    T: Distribution + MaximumInBoundingRegion + AcceptRejectFudge,
{
    type SourceDistribution = T;

    fn simulate(
        &self,
        count: usize,
    ) -> Vec<PointValue<<Self::SourceDistribution as Evaluate>::Output>> {
        let region = self.distribution.bounding_region();
        let mut point_rng = WyRand::new();
        let mut value_rng = WyRand::new();
        iter::repeat_with(|| region.sample(&mut point_rng))
            .map(|pt| self.distribution.evaluate_at_with_probability_density(&pt))
            .filter_map(|(point_value, probability_density)| {
                (probability_density > self.maximum * value_rng.gen_f32()).then_some(point_value)
            })
            .take(count)
            .collect()
    }
}
