//! Library for computing and working with hydrogenic wavefunctions.
//!
//! # Examples
//!
//! To evaluate a wavefunction at a particular point:
//! ```
//! use evanescence_core::geometry::Point;
//! use evanescence_core::numerics::Evaluate;
//! use evanescence_core::orbital::{Qn, Real};
//!
//! // The 4d_{z^2} orbital.
//! let qn = Qn::new(4, 2, 0).unwrap(); // The constructor validates the parameters.
//! let value = Real::new(qn).evaluate(&Point::new(1.0, 3.2, 4.7));
//! approx::assert_ulps_eq!(value, 0.008895547);
//! ```
//!
//! To run a Monte Carlo simulation on an orbital:
//! ```
//! use evanescence_core::numerics::monte_carlo::accept_reject::AcceptReject;
//! use evanescence_core::numerics::monte_carlo::MonteCarlo;
//! use evanescence_core::orbital::{Qn, Real};
//!
//! let qn = Qn::new(4, 2, 0).unwrap();
//! let results = AcceptReject::new(Real::new(qn)).simulate(10_000);
//! ```

#![feature(array_windows, is_sorted, once_cell)]
#![warn(clippy::pedantic, clippy::integer_division)]
#![allow(
    clippy::manual_assert, // Triggered by approx.
    clippy::module_name_repetitions, // Can be necessary for disambiguation.
    clippy::must_use_candidate, // Annoying.
    clippy::non_ascii_literal, // Unicode support is expected.
)]
// FIXME: Treat numerical precision more rigorously.
#![allow(clippy::cast_possible_wrap, clippy::cast_precision_loss)]
// Machine-generated values.
#![cfg_attr(test, allow(clippy::excessive_precision, clippy::unreadable_literal))]
// Proper error handling.
#![cfg_attr(not(test), deny(clippy::unwrap_used))]

#[macro_use]
pub mod numerics;

pub mod geometry;
pub mod orbital;
