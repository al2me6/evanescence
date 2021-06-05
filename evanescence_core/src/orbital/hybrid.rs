//! Implementation of hybrid orbitals.
use std::collections::HashMap;
use std::{fmt, iter, ops};

use approx::relative_eq;
use getset::{CopyGetters, Getters};
use itertools::Itertools;
use thiserror::Error;

use super::quantum_numbers::Qn;
use super::{EvaluateBounded, Orbital, Real};
use crate::geometry::Point;
use crate::numerics::Evaluate;

/// Structure representing a [`Qn`] and an associated weight.
#[derive(Clone, PartialEq, Debug)]
pub struct QnWeight {
    pub qn: Qn,
    pub weight: f32,
}

/// Structure representing a linear combination of multiple orbitals of the same atom.
///
/// # Invariants
/// The orbital forming a `LinearCombination` must be normalized.
#[derive(Clone, PartialEq, Debug)]
pub struct LinearCombination {
    /// The individual orbitals and weights comprising the linear combination.
    combination: Vec<QnWeight>,
}

impl LinearCombination {
    /// Validate that a given set of weights produces a normalized orbital.
    fn validate(weights: impl Iterator<Item = f32>) -> bool {
        relative_eq!(weights.map(|w| w * w).sum::<f32>(), 1.0)
    }

    /// Construct a new linear combination given a set of orbitals, their weights, and a string
    /// describing the combination's kind. This function returns `None` if the resultant mixture
    /// is not normalized.
    pub fn new(combination: Vec<QnWeight>) -> Option<Self> {
        if Self::validate(combination.iter().map(|&QnWeight { weight, .. }| weight)) {
            Some(Self { combination })
        } else {
            None
        }
    }

    /// Iterate over the individual orbitals and weights.
    pub fn iter(&self) -> impl Iterator<Item = &QnWeight> {
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
            let QnWeight { qn, weight } = combination
                .next()
                .expect("linear combination cannot ever be empty");
            Self::format_orbital_weight(*weight, qn)
        })
        .chain(combination.map(|QnWeight { qn, weight }| {
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

impl ops::Index<usize> for LinearCombination {
    type Output = QnWeight;

    fn index(&self, index: usize) -> &Self::Output {
        &self.combination[index]
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
        $crate::orbital::hybrid::LinearCombination::new(
            vec![
                $($crate::orbital::hybrid::QnWeight {
                    qn: $crate::orbital::Qn::new($n, $l, $m).expect("invalid quantum numbers"),
                    weight: $overall_factor * $weight,
                }),+
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
            .map(|QnWeight { qn, weight }| weight * Real::evaluate(qn, point))
            .sum()
    }
}

impl EvaluateBounded for Hybrid {
    fn bound(params: &Self::Parameters) -> f32 {
        params
            .iter()
            .map(|QnWeight { qn, .. }| qn)
            .map(Real::bound)
            .reduce(f32::max)
            .expect("linear combination must contain at least one orbital")
            * 0.9
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

/// Mapping describing how many orbitals of each azimuthal quantum number `l` is contained in a
/// [`Kind`].
pub type Mixture = HashMap<u32, u32>;

/// A structure describing all hybrid orbitals of a certain kind.
///
/// # Invariants
///
/// The following are expected of a valid `Kind`:
///
/// * The mixture is not empty.
/// * The number of orbitals, as indicated by the sum of the number of orbitals mixed, is
///   to the number of [`LinearCombination`]s provided.
/// * The value of `n` is valid: It is nonzero and greater than the `l` values specified in
///   the `mixture`.
/// * All orbitals in linear combinations have the expected value of `n`.
#[derive(Clone, PartialEq, Debug, Getters, CopyGetters)]
pub struct Kind {
    #[getset(get_copy = "pub")]
    /// The principal quantum number `n` of all orbitals.
    n: u32,
    #[getset(get = "pub")]
    /// The number and type of orbitals (by azimuthal quantum number `l`) mixed to form this kind.
    mixture: Mixture,
    #[getset(get = "pub")]
    /// String describing the type of symmetry this kind possesses.
    symmetry: String,
    #[getset(get = "pub")]
    /// Extra information about this kind.
    description: Option<String>,
    #[getset(get = "pub")]
    /// The linear combinations forming this kind.
    combinations: Vec<LinearCombination>,
}

/// Error type describing invalid values passed to [`Kind`]'s constructor.
#[derive(PartialEq, Debug, Error)]
pub enum InvalidKindError {
    /// Kind is empty.
    #[error("kind cannot be empty")]
    Empty,
    /// Kind has the incorrect number of linear combinations.
    #[error("expected {expected} linear combinations from mixture type, got {actual}")]
    IncorrectLength { expected: usize, actual: usize },
    /// Kind has an invalid value of `n` (zero or too small for the `l` values declared).
    #[error("got invalid value of n: {0}")]
    InvalidN(u32),
    /// Kind contained an orbital with a value of `n` different than declared.
    #[error("got value of n that differs from the specified value: {0}")]
    UnexpectedN(u32),
}

impl Kind {
    /// Construct a new `Kind`.
    ///
    /// # Errors
    /// This constructor will return an [error](InvalidKindError) if the invariants of [`Kind`]
    /// are not satisfied.
    pub fn new(
        n: u32,
        mixture: Mixture,
        symmetry: String,
        description: Option<String>,
        combinations: Vec<LinearCombination>,
    ) -> Result<Self, InvalidKindError> {
        if mixture.keys().count() == 0 || combinations.is_empty() {
            return Err(InvalidKindError::Empty);
        }
        let expected_lc_count = mixture.values().sum::<u32>() as usize;
        if expected_lc_count != combinations.len() {
            return Err(InvalidKindError::IncorrectLength {
                expected: expected_lc_count,
                actual: combinations.len(),
            });
        }
        if n == 0 || n <= *mixture.keys().max().expect("mixture is not empty") {
            return Err(InvalidKindError::InvalidN(n));
        }
        for combination in &combinations {
            for QnWeight { qn, .. } in combination.iter() {
                if qn.n() != n {
                    return Err(InvalidKindError::UnexpectedN(qn.n()));
                }
            }
        }
        Ok(Self {
            n,
            mixture,
            symmetry,
            description,
            combinations,
        })
    }

    /// Give the number of linear combinations contained in this kind.
    pub fn count(&self) -> usize {
        self.combinations.len()
    }

    /// Give the first linear combination in this kind.
    pub fn archetype(&self) -> &LinearCombination {
        &self.combinations[0]
    }

    /// Return an iterator over all the linear combinations contained.
    pub fn iter(&self) -> impl Iterator<Item = &LinearCombination> {
        self.combinations.iter()
    }

    /// Give the name of the mixture, ex. `sp³`, where superscripts are represented in Unicode.
    pub fn mixture_name(&self) -> String {
        let mut kind = String::with_capacity(10);
        for (&l, &count) in self.mixture.iter().sorted_by_key(|(&l, _)| l) {
            kind.push_str(super::subshell_name(l).unwrap_or("<?>"));
            if count != 1 {
                count
                    .to_string()
                    .chars()
                    .map(|c| match c {
                        '0' => '⁰',
                        '1' => '¹',
                        '2' => '²',
                        '3' => '³',
                        '4' => '⁴',
                        '5' => '⁵',
                        '6' => '⁶',
                        '7' => '⁷',
                        '8' => '⁸',
                        '9' => '⁹',
                        _ => unreachable!("representation of a `u32` can only contain `[0-9]`"),
                    })
                    .for_each(|c| kind.push(c));
            }
        }
        kind
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref desc) = self.description {
            write!(f, "{} ({})", self.mixture_name(), desc)
        } else {
            write!(f, "{}", self.mixture_name())
        }
    }
}

#[doc(hidden)]
pub mod __private {
    pub use maplit::hashmap;
}

/// Construct a new [`Kind`].
///
/// # Panics
/// This macro will panic if `Kind`'s invariants are not upheld.
#[macro_export]
macro_rules! kind {
    (
        mixture: {
            n: $n:literal,
            $($l:literal => $count:literal),+ $(,)?
        },
        symmetry: $symmetry:literal,
        $(description: $description:literal,)?
        combinations: { $($lc:expr),+ $(,)? } $(,)?
    ) => {
        $crate::orbital::hybrid::Kind::new(
            $n,
            $crate::orbital::hybrid::__private::hashmap! {
                $($l => $count),+
            },
            $symmetry.to_owned(),
            kind!(@desc $($description)?),
            vec![
                $($lc),+
            ],
        )
        .unwrap_or_else(|err| panic!("attempted to create invalid `Kind`: {}", err))
    };
    (@desc $some:literal) => { std::option::Option::Some($some.to_owned()) };
    (@desc) => { std::option::Option::None };
}
