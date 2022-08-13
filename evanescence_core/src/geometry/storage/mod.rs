use super::point::IPoint;

pub mod component_form_3;
pub mod grid_values_3;

pub use component_form_3::ComponentForm3;
pub use grid_values_3::GridValues3;

/// A point and the value of a function evaluated at that point.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, Debug)]
pub struct PointValue<const N: usize, P: IPoint<N>, V>(pub P, pub V);
