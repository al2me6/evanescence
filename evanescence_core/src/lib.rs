//! Library for computing and working with hydrogenic wavefunctions.
//!
//! # Examples
//!
//! To evaluate a wavefunction at a particular point:
//! ```
//! use evanescence_core::geometry::Point;
//! use evanescence_core::numerics::Evaluate;
//! use evanescence_core::orbital::{self, Qn};
//!
//! // The 4d_{z^2} orbital.
//! let qn = Qn::new(4, 2, 0).unwrap(); // The constructor validates the parameters.
//! let value = orbital::Real::evaluate(&qn, &Point::new(1.0, 3.2, 4.7));
//! approx::assert_relative_eq!(value, 0.008895547);
//! ```
//!
//! To run a [Monte Carlo simulation](monte_carlo) on an orbital:
//! ```
//! use evanescence_core::monte_carlo::{MonteCarlo, Quality};
//! use evanescence_core::orbital::{self, Qn};
//!
//! let qn = Qn::new(4, 2, 0).unwrap();
//! let quality = Quality::Low; // Quality controls the number of points sampled.
//! let results = orbital::Real::monte_carlo_simulate(&qn, quality, false);
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

#[macro_use]
pub mod numerics;

pub mod geometry;
pub mod monte_carlo;
pub mod orbital;

pub(crate) mod utils {
    /// Generated an `f32` value in the range \[0, 1\) from a source `u32` value.
    ///
    /// Reproduced from <https://docs.rs/oorandom/11.1.3/src/oorandom/lib.rs.html#104-117>.
    #[inline]
    pub(crate) fn reinterpret_as_f32(mut u: u32) -> f32 {
        const TOTAL_BITS: u32 = 32;
        const PRECISION: u32 = f32::MANTISSA_DIGITS + 1;
        const MANTISSA_SCALE: f32 = 1.0 / ((1_u32 << PRECISION) as f32);
        u >>= TOTAL_BITS - PRECISION;
        u as f32 * MANTISSA_SCALE
    }

    /// Generated a random `f32` value in the range \[0, 1\).
    ///
    /// This is somehow faster when implemented as a macro wrapper around a function...
    #[macro_export]
    macro_rules! rand_f32 {
        ($rng:expr) => {{
            use nanorand::RNG;
            $crate::utils::reinterpret_as_f32($rng.generate::<u32>())
        }};
    }
}
