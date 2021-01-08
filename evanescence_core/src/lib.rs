//! Library for computing and working with hydrogenic wavefunctions.
//!
//! # Examples
//!
//! To evaluate a wavefunction at a particular point:
//! ```
//! use evanescence_core::geometry::Point;
//! use evanescence_core::numerics::Evaluate;
//! use evanescence_core::orbital::{self, QuantumNumbers};
//!
//! // The 4d_{z^2} orbital.
//! let qn = QuantumNumbers::new(4, 2, 0).unwrap(); // The constructor validates the parameters.
//! let value = orbital::Real::evaluate(qn, &Point::new(1.0, 3.2, 4.7));
//! approx::assert_relative_eq!(value, 0.008895547);
//! ```
//!
//! To run a [Monte Carlo simulation](monte_carlo) on an orbital:
//! ```
//! use evanescence_core::monte_carlo::{MonteCarlo, Quality};
//! use evanescence_core::orbital::{self, QuantumNumbers};
//!
//! let qn = QuantumNumbers::new(4, 2, 0).unwrap();
//! let quality = Quality::Low; // Quality controls the number of points sampled.
//! let results = orbital::Real::monte_carlo_simulate(qn, quality);
//! ```

#![feature(destructuring_assignment)]
#![warn(clippy::pedantic, clippy::integer_division)]
#![allow(
    clippy::cast_lossless, // Stylistic.
    clippy::cast_possible_wrap, // We generally work with smaller values, so this should not be a concern.
    clippy::cast_precision_loss, // As with above.
    clippy::excessive_precision, // We have many machine-generated values that are not worth fixing.
    clippy::explicit_iter_loop, // Stylistic.
    clippy::must_use_candidate, // Annoying.
    clippy::non_ascii_literal, // It's 2021... Unicode support is expected.
    clippy::unreadable_literal, // We have many machine-generated values that are not worth fixing.
)]

pub mod geometry;
pub mod monte_carlo;
#[macro_use]
pub mod numerics;
pub mod orbital;

pub(crate) mod utils {
    use getrandom::getrandom;
    use oorandom::Rand32;

    pub(crate) fn new_rng() -> Rand32 {
        let mut buf = [0_u8; 8];
        getrandom(&mut buf).unwrap();
        Rand32::new(u64::from_ne_bytes(buf))
    }
}
