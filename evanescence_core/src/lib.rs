#![feature(iterator_fold_self, min_const_generics)]
#![warn(clippy::pedantic, clippy::integer_division)]
#![allow(
    clippy::excessive_precision,
    clippy::cast_lossless,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::non_ascii_literal,
    clippy::unreadable_literal
)]

pub mod geometry;
#[macro_use]
pub mod numerics;
#[macro_use]
pub mod orbital;
pub mod monte_carlo;