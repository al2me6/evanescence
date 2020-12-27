#![feature(min_const_generics)]
#![warn(clippy::pedantic, clippy::integer_division)]
#![allow(
    clippy::excessive_precision,
    clippy::cast_lossless,
    clippy::must_use_candidate,
    clippy::non_ascii_literal,
    clippy::unreadable_literal
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
