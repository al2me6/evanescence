#![recursion_limit = "1024"]
#![feature(default_free_fn)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_precision_loss, // We work with smaller values, so this should not be a concern.
    clippy::default_trait_access, // Triggered by yew's proc macros.
    clippy::derivable_impls, // Some are handwritten for clarity.
    clippy::manual_filter_map, // Semantics.
    clippy::missing_panics_doc,
    clippy::must_use_candidate, // Unnecessary.
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value, // Triggered by wasm-bindgen macro.
    clippy::non_ascii_literal, // Unicode support is expected.
)]

#[macro_use]
pub mod utils;

pub mod components;
pub mod plotly;
pub mod plotters;
pub mod presets;
pub mod state;

/// Maximum value of the principal quantum number `n` that is exposed.
pub const MAX_N: u32 = 12;
