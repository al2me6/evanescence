pub mod grid_values;
pub mod struct_of_arrays;

pub use self::struct_of_arrays::Soa;
use super::point::IPoint;

/// A point and the value of a function evaluated at that point.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, Debug)]
pub struct PointValue<const N: usize, P: IPoint<N>, V>(pub P, pub V);
