use super::statistics::Distribution;
use super::Evaluate;
use crate::geometry::ComponentForm;

pub mod accept_reject;

pub trait MonteCarlo {
    type SourceDistribution: Distribution;

    fn simulate(
        &self,
        count: usize,
    ) -> ComponentForm<<Self::SourceDistribution as Evaluate>::Output>;
}
