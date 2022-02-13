//! Implementations of atomic (real, complex) and hybrid orbitals.
//!
//! Types for quantum numbers are available through the [`quantum_numbers`] module.
pub mod atomic;
pub mod hybrid;
pub mod molecular;
pub mod quantum_numbers;

use std::marker::PhantomData;

pub use self::atomic::{Complex, Real, Real1};
pub use self::quantum_numbers::Qn;
use crate::geometry::Point;
use crate::numerics::{Evaluate, EvaluateBounded};

/// Trait representing a type of wavefunction.
pub trait Orbital: EvaluateBounded {
    /// Give the probability density value corresponding to a certain wavefunction value.
    fn probability_density_of(value: Self::Output) -> f32;

    /// Give the conventional name of an orbital.
    ///
    /// Superscripts are represented using Unicode superscript symbols and subscripts are
    /// represented with the HTML tag `<sub></sub>`.
    fn name(params: &Self::Parameters) -> String;
}

/// Type that evaluates the probability density of an [`Orbital`].
///
/// Example:
///
/// ```
/// use approx::assert_relative_eq;
/// use evanescence_core::geometry::Point;
/// use evanescence_core::numerics::Evaluate;
/// use evanescence_core::orbital::{ProbabilityDensity, Qn, Real1};
///
/// let qn = Qn::new(3, 2, 1).unwrap();
///
/// assert_relative_eq!(
///     2.446E-4,
///     ProbabilityDensity::<Real1>::evaluate(&qn, &Point::new(6.0, -0.3, 8.5))
/// );
/// ```
pub struct ProbabilityDensity<O>(PhantomData<O>);

impl<O: Orbital> Evaluate for ProbabilityDensity<O> {
    type Output = f32;
    type Parameters = O::Parameters;

    #[inline]
    fn evaluate(params: &Self::Parameters, point: &Point) -> Self::Output {
        O::probability_density_of(O::evaluate(params, point))
    }
}

impl<O: Orbital> EvaluateBounded for ProbabilityDensity<O> {
    #[inline]
    fn bound(params: &Self::Parameters) -> f32 {
        O::bound(params)
    }
}
