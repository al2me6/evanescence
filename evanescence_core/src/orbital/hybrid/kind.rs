use std::collections::HashMap;
use std::fmt;

use getset::{CopyGetters, Getters, Setters};
use itertools::Itertools;
pub use maplit::hashmap;

use super::linear_combination::{Component, LinearCombination};
use crate::orbital::atomic;
use crate::utils::sup_sub_string::{SupSubFormat, SupSubString};

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
#[derive(Clone, PartialEq, Debug, Getters, CopyGetters, Setters)]
pub struct Kind {
    #[getset(get_copy = "pub")]
    /// The principal quantum number `n` of all orbitals.
    pub(crate) n: u32,
    #[getset(get = "pub")]
    /// The number and type of orbitals (by azimuthal quantum number `l`) mixed to form this kind.
    pub(crate) mixture: Mixture,
    #[getset(get = "pub")]
    /// String describing the type of symmetry this kind possesses.
    pub(crate) symmetry: String,
    #[getset(get = "pub", set = "pub")]
    /// Extra information about this kind.
    pub(crate) description: Option<String>,
    #[getset(get = "pub")]
    /// The linear combinations forming this kind.
    pub(crate) combinations: Vec<LinearCombination>,
}

/// Error type describing invalid values passed to [`Kind`]'s constructor.
#[derive(PartialEq, Eq, Debug, thiserror::Error)]
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
            kind.push_nrm(atomic::subshell_name(l).unwrap_or("<?>"));
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
        $crate::orbital::hybrid::kind::Kind::new(
            $n,
            $crate::orbital::hybrid::kind::hashmap! {
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
