//! Implementations of atomic (real, complex) and hybrid orbitals.
//!
//! Types for quantum numbers are available through the [`quantum_numbers`] module.
pub mod atomic;
pub mod hybrid;
pub mod quantum_numbers;

pub use self::atomic::complex::Complex as AtomicComplex;
pub use self::atomic::real::Real as AtomicReal;
pub use self::quantum_numbers::Qn;
use crate::geometry::point::IPoint;
use crate::numerics::statistics::Distribution;
use crate::utils::sup_sub_string::SupSubString;

/// Trait representing a type of wavefunction.
pub trait Orbital<P: IPoint<3>>: Distribution<3, P> {
    /// Give the conventional name of an orbital.
    fn name(&self) -> SupSubString;
}
