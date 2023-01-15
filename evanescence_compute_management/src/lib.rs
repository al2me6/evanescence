#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]

extern crate nalgebra as na;

pub mod computation;
pub mod computation_host;
pub mod evaluator;

pub use computation::ExecuteComputation;
pub use computation_host::ComputationHost;
