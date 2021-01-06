#![feature(destructuring_assignment)]
#![warn(clippy::pedantic, clippy::integer_division)]
#![allow(
    clippy::cast_lossless, // Stylistic.
    clippy::cast_possible_wrap, // We generally work with smaller values, so this should not be a concern.
    clippy::cast_precision_loss, // As with above.
    clippy::excessive_precision, // We have many machine-generated values that are not worth fixing.
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
    use oorandom::Rand64;

    pub(crate) fn new_rng() -> Rand64 {
        let mut buf = [0_u8; 16];
        getrandom(&mut buf).unwrap();
        Rand64::new(u128::from_ne_bytes(buf))
    }
}
