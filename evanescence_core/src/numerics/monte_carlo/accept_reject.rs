use std::iter;

use na::Point;

use super::MonteCarlo;
use crate::geometry::point::IPoint;
use crate::geometry::region::{BoundingRegion, Region};
use crate::geometry::storage::PointValue;
use crate::numerics::random::WyRand;
use crate::numerics::statistics::Distribution;

pub trait AcceptRejectParameters<const N: usize, P: IPoint<N> = Point<f32, N>>:
    BoundingRegion<N, P> + Distribution<N, P>
{
    fn maximum(&self) -> f32 {
        let mut rng = WyRand::new();
        let region = self.bounding_region();
        Iterator::chain(
            iter::once(P::origin()),
            iter::repeat_with(|| region.sample(&mut rng)),
        )
        .take(200_000)
        .map(|pt| self.probability_density(&pt))
        .reduce(f32::max)
        .unwrap()
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
    rng: WyRand,
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
            rng: WyRand::new(),
        }
    }
}

impl<const N: usize, P, D> Iterator for AcceptReject<N, P, D>
where
    P: IPoint<N>,
    D: AcceptRejectParameters<N, P>,
{
    type Item = PointValue<N, P, D::Output>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let point = self.region.sample(&mut self.rng);
            let (point_value, probability_density) = self
                .distribution
                .evaluate_at_with_probability_density(point);
            if probability_density > self.maximum * self.rng.gen_f32() {
                return Some(point_value);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

impl<const N: usize, P, D> MonteCarlo<N, P> for AcceptReject<N, P, D>
where
    P: IPoint<N>,
    D: AcceptRejectParameters<N, P>,
{
    type Output = D::Output;
}
