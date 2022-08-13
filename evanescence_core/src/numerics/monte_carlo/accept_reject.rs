use std::iter;
use std::marker::PhantomData;

use super::MonteCarlo;
use crate::geometry::point::IPoint;
use crate::geometry::region::{BoundingRegion, Region};
use crate::geometry::storage::PointValue;
use crate::numerics::random::WyRand;
use crate::numerics::statistics::Distribution;

pub trait AcceptRejectParameters<const N: usize, P: IPoint<N>>:
    BoundingRegion<N, P> + Distribution<N, P>
{
    fn maximum(&self) -> f32 {
        let mut rng = WyRand::new();
        let region = self.bounding_region();
        // TODO: sample adaptively?
        Iterator::chain(
            iter::once(P::origin()),
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

#[derive(Clone)]
pub struct AcceptReject<const N: usize, P, D>
where
    P: IPoint<N>,
    D: AcceptRejectParameters<N, P>,
{
    distribution: D,
    region: <D as BoundingRegion<N, P>>::Geometry,
    maximum: f32,
    point_rng: WyRand,
    value_rng: WyRand,
    _phantom: PhantomData<P>,
}

impl<const N: usize, P, D> AcceptReject<N, P, D>
where
    P: IPoint<N>,
    D: AcceptRejectParameters<N, P>,
{
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
            _phantom: PhantomData,
        }
    }
}

impl<const N: usize, P, D> MonteCarlo<N, P> for AcceptReject<N, P, D>
where
    P: IPoint<N>,
    D: AcceptRejectParameters<N, P>,
{
    type Output = D::Output;

    fn simulate(&mut self, count: usize) -> Vec<PointValue<N, P, Self::Output>> {
        iter::repeat_with(|| self.region.sample(&mut self.point_rng))
            .map(|pt| self.distribution.evaluate_at_with_probability_density(pt))
            .filter_map(|(point_value, probability_density)| {
                (probability_density > self.maximum * self.value_rng.gen_f32())
                    .then_some(point_value)
            })
            .take(count)
            .collect()
    }
}
