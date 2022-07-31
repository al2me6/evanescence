//! Implementations of atomic (real, complex) and hybrid orbitals.
//!
//! Types for quantum numbers are available through the [`quantum_numbers`] module.
pub mod atomic;
pub mod hybrid;
pub mod monte_carlo;
pub mod quantum_numbers;

pub use self::atomic::complex::Complex;
pub use self::atomic::real::Real;
pub use self::quantum_numbers::Qn;
use crate::geometry::Point;
use crate::numerics::{Evaluate, EvaluateBounded};

/// Trait representing a type of wavefunction.
pub trait Orbital: EvaluateBounded {
    /// Give the probability density value corresponding to a certain wavefunction value.
    fn probability_density_of(&self, value: Self::Output) -> f32;

    /// Give the conventional name of an orbital.
    ///
    /// Superscripts are represented using Unicode superscript symbols and subscripts are
    /// represented with the HTML tag `<sub></sub>`.
    fn name(&self) -> String;
}

/// Type that evaluates the probability density of an [`Orbital`].
///
/// Example:
///
/// ```
/// use approx::assert_relative_eq;
/// use evanescence_core::geometry::Point;
/// use evanescence_core::numerics::Evaluate;
/// use evanescence_core::orbital::{ProbabilityDensity, Qn, Real};
///
/// let qn = Qn::new(3, 2, 1).unwrap();
///
/// assert_relative_eq!(
///     2.446E-4,
///     ProbabilityDensity::new(Real::new(qn)).evaluate(&Point::new(6.0, -0.3, 8.5))
/// );
/// ```
pub struct ProbabilityDensity<O>(O);

impl<O> ProbabilityDensity<O> {
    pub fn new(inner: O) -> Self {
        Self(inner)
    }
}

impl<O: Orbital> Evaluate for ProbabilityDensity<O> {
    type Output = f32;

    #[inline]
    fn evaluate(&self, point: &Point) -> Self::Output {
        self.0.probability_density_of(self.0.evaluate(point))
    }
}

impl<O: Orbital> EvaluateBounded for ProbabilityDensity<O> {
    #[inline]
    fn bound(&self) -> f32 {
        self.0.bound()
    }
}

impl<T: Copy> Evaluate for ProbabilityDensity<Box<dyn Orbital<Output = T>>> {
    type Output = f32;

    #[inline]
    fn evaluate(&self, point: &Point) -> Self::Output {
        self.0.probability_density_of(self.0.evaluate(point))
    }
}

impl<T: Copy> EvaluateBounded for ProbabilityDensity<Box<dyn Orbital<Output = T>>> {
    #[inline]
    fn bound(&self) -> f32 {
        self.0.bound()
    }
}
