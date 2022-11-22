use std::f32::consts::FRAC_1_SQRT_2;
use std::{fmt, ops};

use approx::relative_eq;

use super::super::{AtomicReal, Qn};
use crate::numerics::consts::{FRAC_1_SQRT_3, FRAC_1_SQRT_6};
use crate::utils::sup_sub_string::{SupSubFormat, SupSubString};

/// Structure representing a [`Qn`] and an associated weight.
#[derive(Clone, PartialEq, Debug)]
pub struct Component {
    pub qn: Qn,
    pub weight: f32,
}

/// Structure representing a linear combination of multiple orbitals of the same atom.
///
/// # Invariants
/// The orbital forming a `LinearCombination` must be normalized.
#[derive(Clone, PartialEq, Debug)]
pub struct LinearCombination(Vec<Component>);

impl LinearCombination {
    /// Validate that a given set of weights produces a normalized orbital.
    pub(crate) fn validate(weights: impl Iterator<Item = f32>) -> bool {
        relative_eq!(weights.map(|w| w * w).sum::<f32>(), 1.0)
    }

    /// Construct a new linear combination given a set of orbitals, their weights, and a string
    /// describing the combination's kind. This function returns `None` if the resultant mixture
    /// is not normalized.
    pub fn new(combination: Vec<Component>) -> Option<Self> {
        if Self::validate(combination.iter().map(|&Component { weight, .. }| weight)) {
            Some(Self(combination))
        } else {
            None
        }
    }

    /// Iterate over the individual orbitals and weights.
    pub fn iter(&self) -> impl Iterator<Item = &Component> {
        self.0.iter()
    }

    /// Get the number of orbitals this linear combination is composed of.
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// Pretty-print a single orbital and its weight.
    pub(crate) fn format_orbital_weight(weight: f32, qn: Qn) -> SupSubString {
        // Try to express exact values as such.
        // FIXME: This is a rather ad-hoc and fragile way to do this.
        pub(crate) const EXACT_VALUES: [(f32, &str); 6] = [
            (0.288_675_1, "1/√12"),
            (FRAC_1_SQRT_6, "1/√6"),
            (0.5, "1/2"),
            (FRAC_1_SQRT_3, "1/√3"),
            (FRAC_1_SQRT_2, "1/√2"),
            (0.816_496_6, "√(2/3)"),
        ];
        let coefficient = EXACT_VALUES
            .iter()
            .find_map(|&(val, expr)| {
                approx::relative_eq!(val, weight.abs(), max_relative = 1E-6).then(|| {
                    if weight > 0.0 {
                        expr.to_owned()
                    } else {
                        format!("−{expr}")
                    }
                })
            })
            .unwrap_or_else(|| format!("{weight:.3}").trim_end_matches('0').to_owned());
        let mut ret = sup_sub_string![nrm(coefficient) " "];
        ret.extend(AtomicReal::name_qn(qn));
        ret
    }

    /// Give an expression describing the linear combination. (Ex. `0.707 2s + 0.707 2p_z`).
    pub fn expression(&self) -> SupSubString {
        self.iter()
            .enumerate()
            .map(|(i, &Component { qn, weight })| {
                if i == 0 {
                    Self::format_orbital_weight(weight, qn)
                } else {
                    // Manually add signs to add padding.
                    let mut component =
                        sup_sub_string![nrm(if weight < 0.0 { " − " } else { " + " })];
                    // Take the absolute value since the sign was added manually.
                    component.extend(Self::format_orbital_weight(weight.abs(), qn));
                    component
                }
            })
            .collect()
    }
}

impl fmt::Display for LinearCombination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.expression().format_or_normal(SupSubFormat::Unicode))
    }
}

impl ops::Index<usize> for LinearCombination {
    type Output = Component;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

/// Construct a new [`LinearCombination`].
///
/// Example:
///
/// ```
/// # use evanescence_core::lc;
/// let linear_combination = lc! {
///     overall: std::f32::consts::FRAC_1_SQRT_2, // Overall factor.
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
        overall: $overall_factor:expr,
        $(($n:literal, $l:literal, $m:literal) * $weight:expr),+
        $(,)?
    ) => {
        $crate::orbital::hybrid::linear_combination::LinearCombination::new(
            vec![
                $($crate::orbital::hybrid::linear_combination::Component {
                    qn: $crate::orbital::Qn::new($n, $l, $m).expect("invalid quantum numbers"),
                    weight: $overall_factor * $weight,
                }),+
            ],
        )
        .expect("linear combination is not normalized")
    };
}
