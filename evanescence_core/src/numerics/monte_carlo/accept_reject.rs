use std::iter;

use super::MonteCarlo;
use crate::geometry::region::{BoundingRegion, Region};
use crate::geometry::{Point, PointValue};
use crate::numerics::random::WyRand;
use crate::numerics::statistics::Distribution;
use crate::numerics::Evaluate;

pub trait AcceptRejectParameters: BoundingRegion + Distribution {
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

    fn accept_threshold_fudge(&self) -> Option<f32> {
        None
    }
}

pub struct AcceptReject<D: AcceptRejectParameters> {
    distribution: D,
    region: <D as BoundingRegion>::Geometry,
    maximum: f32,
    point_rng: WyRand,
    value_rng: WyRand,
}

impl<D: AcceptRejectParameters> AcceptReject<D> {
    pub fn new(distribution: D) -> Self {
        let mut maximum = distribution.maximum();
        if let Some(modifier) = distribution.accept_threshold_fudge() {
            maximum *= modifier;
        }
        let region = distribution.bounding_region();
        Self {
            distribution,
            region,
            maximum,
            point_rng: WyRand::new(),
            value_rng: WyRand::new(),
        }
    }
}

impl<D: AcceptRejectParameters> MonteCarlo for AcceptReject<D> {
    type SourceDistribution = D;

    fn simulate(
        &mut self,
        count: usize,
    ) -> Vec<PointValue<<Self::SourceDistribution as Evaluate>::Output>> {
        iter::repeat_with(|| self.region.sample(&mut self.point_rng))
            .map(|pt| self.distribution.evaluate_at_with_probability_density(&pt))
            .filter_map(|(point_value, probability_density)| {
                (probability_density > self.maximum * self.value_rng.gen_f32())
                    .then_some(point_value)
            })
            .take(count)
            .collect()
    }
}
