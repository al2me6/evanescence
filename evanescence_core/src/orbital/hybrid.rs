//! Implementation of hybrid orbitals.
use std::collections::HashMap;
use std::f32::consts::FRAC_1_SQRT_2;
use std::{fmt, ops};

use approx::relative_eq;
use getset::{CopyGetters, Getters};
use itertools::Itertools;
pub use maplit::hashmap;
use thiserror::Error;

use super::{AtomicReal, Orbital, Qn};
use crate::geometry::point::SphericalPoint3;
use crate::geometry::region::{BallCenteredAtOrigin, BoundingRegion};
use crate::numerics::consts::{FRAC_1_SQRT_3, FRAC_1_SQRT_6};
use crate::numerics::monte_carlo::accept_reject::AcceptRejectParameters;
use crate::numerics::statistics::Distribution;
use crate::numerics::Function;
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
pub struct LinearCombination {
    /// The individual orbitals and weights comprising the linear combination.
    combination: Vec<Component>,
}

impl LinearCombination {
    /// Validate that a given set of weights produces a normalized orbital.
    fn validate(weights: impl Iterator<Item = f32>) -> bool {
        relative_eq!(weights.map(|w| w * w).sum::<f32>(), 1.0)
    }

    /// Construct a new linear combination given a set of orbitals, their weights, and a string
    /// describing the combination's kind. This function returns `None` if the resultant mixture
    /// is not normalized.
    pub fn new(combination: Vec<Component>) -> Option<Self> {
        if Self::validate(combination.iter().map(|&Component { weight, .. }| weight)) {
            Some(Self { combination })
        } else {
            None
        }
    }

    /// Iterate over the individual orbitals and weights.
    pub fn iter(&self) -> impl Iterator<Item = &Component> {
        self.combination.iter()
    }

    /// Get the number of orbitals this linear combination is composed of.
    pub fn count(&self) -> usize {
        self.combination.len()
    }

    /// Pretty-print a single orbital and its weight.
    fn format_orbital_weight(weight: f32, qn: Qn) -> SupSubString {
        // Try to express exact values as such.
        // FIXME: This is a rather ad-hoc and fragile way to do this.
        const EXACT_VALUES: [(f32, &str); 6] = [
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
            .unwrap_or_else(|| format!("{:.3}", weight).trim_end_matches('0').to_owned());
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
        f.write_str(&self.expression().format(SupSubFormat::Unicode).unwrap())
    }
}

impl ops::Index<usize> for LinearCombination {
    type Output = Component;

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
                $($crate::orbital::hybrid::Component {
                    qn: $crate::orbital::Qn::new($n, $l, $m).expect("invalid quantum numbers"),
                    weight: $overall_factor * $weight,
                }),+
            ],
        )
        .expect("linear combination is not normalized")
    };
}

/// Implementation of hybrid orbitals.
#[derive(Clone, PartialEq, Debug)]
pub struct Hybrid {
    lc: LinearCombination,
    reals: Vec<AtomicReal>,
}

impl Hybrid {
    pub fn new(lc: LinearCombination) -> Self {
        let reals = lc.iter().map(|comp| AtomicReal::new(comp.qn)).collect();
        Self { lc, reals }
    }
}

impl Function<3, SphericalPoint3> for Hybrid {
    type Output = f32;

    #[inline]
    fn evaluate(&self, point: &SphericalPoint3) -> Self::Output {
        self.lc
            .iter()
            .enumerate()
            .map(|(idx, Component { weight, .. })| weight * self.reals[idx].evaluate(point))
            .sum()
    }
}

impl BoundingRegion<3, SphericalPoint3> for Hybrid {
    type Geometry = BallCenteredAtOrigin;

    fn bounding_region(&self) -> Self::Geometry {
        BallCenteredAtOrigin {
            radius: self
                .reals
                .iter()
                .map(|real| real.bounding_region().radius)
                .reduce(f32::max)
                .expect("linear combination must contain at least one orbital")
                * 0.9,
        }
    }
}

impl Distribution<3, SphericalPoint3> for Hybrid {
    #[inline]
    fn probability_density_of(&self, value: Self::Output) -> f32 {
        value * value
    }
}

impl Orbital<SphericalPoint3> for Hybrid {
    fn name(&self) -> SupSubString {
        self.lc.expression()
    }
}

impl AcceptRejectParameters<3, SphericalPoint3> for Hybrid {}

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
/// * The number of orbitals, as indicated by the sum of the number of orbitals mixed, is equal
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
#[derive(PartialEq, Eq, Debug, Error)]
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
            for Component { qn, .. } in combination.iter() {
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

    /// Give the name of the mixture, ex. `sp^3`.
    pub fn mixture_name(&self) -> SupSubString {
        let mut kind = sup_sub_string![];
        for (&l, &count) in self.mixture.iter().sorted_by_key(|(&l, _)| l) {
            kind.push_nrm(super::atomic::subshell_name(l).unwrap_or("<?>"));
            if count != 1 {
                kind.push_sup(count.to_string());
            }
        }
        kind
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.mixture_name().format(SupSubFormat::Unicode).unwrap())?;
        if let Some(ref desc) = self.description {
            write!(f, " ({desc})")?;
        }
        Ok(())
    }
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
            $crate::orbital::hybrid::hashmap! {
                $($l => $count),+
            },
            $symmetry.to_owned(),
            kind!(@desc $($description)?),
            vec![
                $($lc),+
            ],
        )
        .unwrap_or_else(|err| panic!("attempted to create invalid `Kind`: {err}"))
    };
    (@desc $some:literal) => { std::option::Option::Some($some.to_owned()) };
    (@desc) => { std::option::Option::None };
}
