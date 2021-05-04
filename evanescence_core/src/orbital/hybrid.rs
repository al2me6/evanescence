//! Implementation of hybrid orbitals.
use std::{fmt, iter};

use approx::relative_eq;
use getset::Getters;

use super::quantum_numbers::Qn;
use super::{EvaluateBounded, Orbital, Real};
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
}

impl LinearCombination {
    /// Validate that a given set of weights produces a normalized orbital.
    fn validate(weights: impl Iterator<Item = f32>) -> bool {
        relative_eq!(weights.map(|w| w * w).sum::<f32>(), 1.0)
    }

    /// Construct a new linear combination given a set of orbitals, their weights, and a string
    /// describing the combination's kind. This function returns `None` if the resultant mixture
    /// is not normalized.
    pub fn new(combination: Vec<(Qn, f32)>) -> Option<Self> {
        if Self::validate(combination.iter().map(|(_, weight)| *weight)) {
            Some(Self { combination })
        } else {
            None
        }
    }

    /// Iterate over the individual orbitals and weights.
    pub fn iter(&self) -> impl Iterator<Item = &(Qn, f32)> {
        self.combination.iter()
    }

    /// Get the number of orbitals this linear combination is composed of.
    pub fn count(&self) -> usize {
        self.combination.len()
    }

    /// Pretty-print a single orbital and its weight.
    fn format_orbital_weight(weight: f32, qn: &Qn) -> String {
        format!(
            "{} {}",
            format!("{:.3}", weight).trim_end_matches('0'),
            Real::name(qn)
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
        f.write_str(&self.expression())
    }
}

/// Construct a new [`LinearCombination`]:
///
/// ```
/// # use evanescence_core::lc;
/// let linear_combination = lc! {
///     overall = std::f32::consts::FRAC_1_SQRT_2, // Overall factor.
///     (2, 0, 0) * 1.0, // Quantum numbers and their associated weights.
///     (2, 1, 0) * 1.0,
/// };
/// ```
///
/// # Panics
/// This macro will panic at runtime if the quantum numbers passed are invalid, or if the
/// resultant linear combination is not normalized.
#[macro_export]
macro_rules! lc {
    (
        overall = $overall_factor:expr,
        $(($n:literal, $l:literal, $m:literal) * $weight:expr),+
        $(,)?
    ) => {
        $crate::orbital::hybrid::LinearCombination::new(
            vec![
                $((
                    $crate::orbital::Qn::new($n, $l, $m).expect("invalid quantum numbers"),
                    $overall_factor * $weight,
                )),+
            ],
        )
        .expect("linear combination is not normalized")
    };
}

/// Implementation of hybrid orbitals.
pub struct Hybrid;

impl Evaluate for Hybrid {
    type Output = f32;
    type Parameters = LinearCombination;

    fn evaluate(combination: &LinearCombination, point: &Point) -> Self::Output {
        combination
            .iter()
            .map(|(qn, weight)| weight * Real::evaluate(qn, point))
            .sum()
    }
}

impl EvaluateBounded for Hybrid {
    fn bound(params: &Self::Parameters) -> f32 {
        params
            .iter()
            .map(|(qn, _)| qn)
            .map(Real::bound)
            .reduce(f32::max)
            .expect("linear combination must contain at least one orbital")
            * 0.8
    }
}

impl Orbital for Hybrid {
    #[inline]
    fn probability_density_of(value: Self::Output) -> f32 {
        value * value
    }

    fn name(params: &Self::Parameters) -> String {
        params.to_string()
    }
}
