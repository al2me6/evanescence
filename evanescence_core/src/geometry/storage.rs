use super::point::IPoint;

pub mod grid_values_3;
pub mod struct_of_arrays;

pub use grid_values_3::GridValues3;
pub use struct_of_arrays::Soa;

/// A point and the value of a function evaluated at that point.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, Debug)]
pub struct PointValue<const N: usize, P: IPoint<N>, V>(pub P, pub V);
