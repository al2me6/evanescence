use super::statistics::Distribution;
use super::Evaluate;
use crate::geometry::PointValue;

pub mod accept_reject;

pub trait MonteCarlo {
    type SourceDistribution: Distribution;

    fn simulate(
        &mut self,
        count: usize,
    ) -> Vec<PointValue<<Self::SourceDistribution as Evaluate>::Output>>;
}
