use crate::geometry::point::{IPoint, SphericalPoint3};
use crate::geometry::storage::PointValue;

pub mod accept_reject;

pub trait MonteCarlo<const N: usize, P: IPoint<N>> {
    type Output: Copy;

    fn simulate(&mut self, count: usize) -> Vec<PointValue<N, P, Self::Output>>;
}

/// Assert dyn-safety.
const _: Option<Box<dyn MonteCarlo<3, SphericalPoint3, Output = f32>>> = None;
