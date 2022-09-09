//! Implementation of hybrid orbitals.

#[macro_use]
pub mod linear_combination;

use std::collections::HashMap;
use std::fmt;

use getset::{CopyGetters, Getters};
use itertools::Itertools;
pub use maplit::hashmap;
use thiserror::Error;

use self::linear_combination::Component;
pub use self::linear_combination::LinearCombination;
use super::{AtomicReal, Orbital};
use crate::geometry::point::SphericalPoint3;
use crate::geometry::region::{BallCenteredAtOrigin, BoundingRegion};
use crate::geometry::storage::PointValue;
use crate::numerics::monte_carlo::accept_reject::AcceptRejectParameters;
use crate::numerics::optimization::simple_x::{BoundingSimplex, Simple};
use crate::numerics::statistics::Distribution;
use crate::numerics::Function;
use crate::utils::sup_sub_string::{SupSubFormat, SupSubString};

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

impl AcceptRejectParameters<3, SphericalPoint3> for Hybrid {
    fn maximum(&self) -> f32 {
        const ITERS: usize = 400;
        const EXPLORATION_PREFERENCE: f32 = 0.3;

        let PointValue(_, max) = Simple::new(
            self.bounding_simplex(),
            |pt| self.evaluate(pt).abs(),
            EXPLORATION_PREFERENCE,
        )
        .maximize(ITERS);

        self.probability_density_of(max)
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

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_1_SQRT_2;
    use std::iter;

    use super::Hybrid;
    use crate::geometry::point::{IPoint, SphericalPoint3};
    use crate::geometry::region::{BoundingRegion, Region};
    use crate::numerics::consts::{FRAC_1_SQRT_6, SQRT_3};
    use crate::numerics::monte_carlo::accept_reject::AcceptRejectParameters;
    use crate::numerics::random::WyRand;
    use crate::numerics::statistics::Distribution;

    #[test]
    fn max_prob_density_computation() {
        const BRUTE_FORCE_SAMPLE_COUNT: usize = 2_000_000;

        let sp3d2 = Hybrid::new(lc! {
            overall: FRAC_1_SQRT_6,
            (3, 0, 0) * 1.0,
            (3, 1, 1) * SQRT_3,
            (3, 2, 0) * -FRAC_1_SQRT_2,
            (3, 2, 2) * SQRT_3 * FRAC_1_SQRT_2,
        });

        let rng = &mut WyRand::new();
        let region = sp3d2.bounding_region();
        let brute_force_max = Iterator::chain(
            iter::once(SphericalPoint3::origin()),
            iter::repeat_with(|| region.sample(rng)),
        )
        .take(BRUTE_FORCE_SAMPLE_COUNT)
        .map(|pt| sp3d2.probability_density(&pt))
        .reduce(f32::max)
        .unwrap();

        let explicit_max = <_ as AcceptRejectParameters<3, _>>::maximum(&sp3d2);

        println!("brute-force: {brute_force_max}; explicit: {explicit_max}");
        assert!(brute_force_max * (1.0 - 1E-6) <= explicit_max);
    }
}
