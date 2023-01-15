#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

extern crate nalgebra as na;

pub mod computation;
pub mod computation_host;
pub mod evaluator;

pub use computation::ExecuteComputation;
pub use computation_host::ComputationHost;
