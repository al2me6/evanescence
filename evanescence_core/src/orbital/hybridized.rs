//! Implementation of hybridized orbitals.
use std::fmt;
use std::iter;

use approx::relative_eq;
use getset::Getters;

use super::quantum_numbers::Qn;
use super::{Orbital, Real as RealOrbital};
use crate::geometry::Point;
use crate::numerics::Evaluate;

/// Structure representing a linear combination of multiple orbitals of the same atom, where
/// each component is stored in a `Vec` as a 2-tuple of the component's quantum numbers and its
/// weight.
///
/// # Invariants
/// The orbital represented by a `LinearCombination` must be normalized.
#[derive(Clone, PartialEq, Debug, Getters)]
pub struct LinearCombination {
    /// The individual orbitals and weights comprising the linear combination.
    combination: Vec<(Qn, f32)>,
    #[getset(get = "pub")]
    /// The kind (ex. "sp³") of mixture that this linear combination is.
    kind: String,
}

impl LinearCombination {
    /// Validate that a given set of weights produces a normalized orbital.
    fn validate(weights: impl Iterator<Item = f32>) -> bool {
        relative_eq!(weights.map(|w| w * w).sum::<f32>(), 1.0)
    }

    /// Construct a new linear combination given a set of orbitals, their weights, and a string
    /// describing the combination's kind. This function returns `None` if the resultant mixture
    /// is not normalized.
    pub fn new(combination: Vec<(Qn, f32)>, kind: String) -> Option<Self> {
        if Self::validate(combination.iter().map(|(_, weight)| *weight)) {
            Some(Self { combination, kind })
        } else {
            None
        }
    }

    /// Iterate over the individual orbitals and weights.
    pub fn iter(&self) -> impl Iterator<Item = &(Qn, f32)> {
        self.combination.iter()
    }

    /// Pretty-print a single orbital and its weight.
    fn format_orbital_weight(weight: f32, qn: &Qn) -> String {
        format!(
            "{} {}",
            format!("{:.3}", weight).trim_end_matches('0'),
            RealOrbital::name(qn)
        )
    }

    /// Give an expression describing the linear combination. (Ex. `0.707 2s + 0.707 2p_z`),
    /// where subscripts are represented with the `<sub></sub>` HTML tag.
    pub fn expression(&self) -> String {
        let mut combination = self.iter();
        iter::once({
            let (qn, weight) = combination
                .next()
                .expect("linear combination cannot ever be empty");
            Self::format_orbital_weight(*weight, qn)
        })
        .chain(combination.map(|(qn, weight)| {
            format!(
                "{sign}{weighted_orbital}",
                // Manually add signs to add padding.
                sign = if *weight < 0.0 { " − " } else { " + " },
                // Note that we take the absolute value since we already manually added the sign.
                weighted_orbital = Self::format_orbital_weight(weight.abs(), qn)
            )
        }))
        .collect()
    }
}

impl fmt::Display for LinearCombination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.kind)
    }
}

/// Construct a new [`LinearCombination`]:
///
/// ```
/// # use evanescence_core::lc;
/// let linear_combination = lc! {
///     kind = "sp",
///     overall = std::f32::consts::FRAC_1_SQRT_2, // Overall factor.
///     (2, 0, 0) => 1.0, // Quantum numbers and their associated weights.
///     (2, 1, 0) => 1.0,
/// };
/// ```
///
/// # Panics
/// This macro will panic at runtime if the quantum numbers passed are invalid, or if the
/// resultant linear combination is not normalized.
#[macro_export]
macro_rules! lc {
    (
        kind = $kind:literal,
        overall = $overall_factor:expr,
        $(($n:literal, $l:literal, $m:literal) => $weight:expr),+
        $(,)?
    ) => {
        $crate::orbital::hybridized::LinearCombination::new(
            vec![
                $((
                    $crate::orbital::Qn::new($n, $l, $m).expect("invalid quantum numbers"),
                    $overall_factor * $weight,
                )),+
            ],
            $kind.to_owned(),
        )
        .expect("linear combination is not normalized")
    };
}

/// Implementation of hybridized orbitals.
pub struct Hybridized;

impl Evaluate for Hybridized {
    type Parameters = LinearCombination;
    type Output = f32;

    fn evaluate(combination: &LinearCombination, point: &Point) -> Self::Output {
        combination
            .iter()
            .map(|(qn, weight)| weight * RealOrbital::evaluate(qn, point))
            .sum()
    }
}

impl Orbital for Hybridized {
    fn estimate_radius(params: &Self::Parameters) -> f32 {
        params
            .iter()
            .map(|(qn, _)| qn)
            .map(RealOrbital::estimate_radius)
            .reduce(f32::max)
            .expect("linear combination must contain at least one orbital")
    }

    fn name(params: &Self::Parameters) -> String {
        params.to_string()
    }
}
