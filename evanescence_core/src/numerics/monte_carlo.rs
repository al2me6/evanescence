use crate::geometry::point::{IPoint, SphericalPoint3};
use crate::geometry::storage::PointValue;

pub mod accept_reject;

pub trait MonteCarlo<const N: usize, P: IPoint<N>>:
    Iterator<Item = PointValue<N, P, Self::Output>>
{
    type Output: Copy;

    /// Convenience method for getting a collection of samples.
    fn simulate(&mut self, count: usize) -> Vec<PointValue<N, P, Self::Output>> {
        self.take(count).collect()
    }
}

/// Convenience type alias to save typing.
pub type DynMonteCarlo<const N: usize, P, O> =
    dyn MonteCarlo<N, P, Output = O, Item = PointValue<N, P, O>> + Send + Sync;

/// Assert dyn-safety.
const _: Option<Box<DynMonteCarlo<3, SphericalPoint3, f32>>> = None;
