//! Library for computing and working with hydrogenic wavefunctions.
//!
//! # Examples
//!
//! To evaluate a wavefunction at a particular point:
//! ```
//! use evanescence_core::geometry::point::SphericalPoint3;
//! use evanescence_core::numerics::Function;
//! use evanescence_core::orbital::AtomicReal;
//! use evanescence_core::qn;
//!
//! // The 4d_{z^2} orbital.
//! let qn = qn!(4, 2, 0); // The constructor validates the parameters.
//! let value = AtomicReal::new(qn).evaluate(&SphericalPoint3::new(1.0, 3.2, 4.7));
//! approx::assert_ulps_eq!(value, 0.008895547);
//! ```
//!
//! To run a Monte Carlo simulation on an orbital:
//! ```
//! use evanescence_core::numerics::monte_carlo::accept_reject::AcceptReject;
//! use evanescence_core::numerics::monte_carlo::MonteCarlo;
//! use evanescence_core::orbital::AtomicReal;
//! use evanescence_core::qn;
//!
//! let qn = qn!(4, 2, 0);
//! let results = AcceptReject::new(AtomicReal::new(qn)).simulate(10_000);
//! ```

#![feature(array_windows, is_sorted, once_cell)]
#![warn(clippy::pedantic, clippy::integer_division)]
#![allow(
    clippy::manual_assert, // Triggered by approx.
    clippy::missing_panics_doc, // Some are internal checks.
    clippy::module_name_repetitions, // Can be necessary for disambiguation.
    clippy::must_use_candidate, // Annoying.
    clippy::non_ascii_literal, // Unicode support is expected.
    clippy::trait_duplication_in_bounds, // False positives.
)]
// FIXME: Treat numerical precision more rigorously.
#![allow(clippy::cast_possible_wrap, clippy::cast_precision_loss)]
// Machine-generated values.
#![cfg_attr(test, allow(clippy::excessive_precision, clippy::unreadable_literal))]

extern crate nalgebra as na;
extern crate typenum as tn;

#[macro_use]
pub mod utils;
#[macro_use]
pub mod numerics;
#[macro_use]
pub mod orbital;

pub mod geometry;
