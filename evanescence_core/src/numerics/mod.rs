//! Functions and traits relating to numerical evaluation.

use std::ops::RangeInclusive;

use itertools::Itertools;

use crate::geometry::Linspace;

/// Verify that two iterables containing float values are approximately equal.
#[cfg(test)]
macro_rules! assert_iterable_relative_eq {
    ($lhs:expr, $rhs: expr $(, $opt:ident = $val:expr)* $(,)?) => {{
        use itertools::Itertools;
        assert!(
            $lhs.iter()
                .zip_eq($rhs.iter())
                .all(|(l, r)| approx::relative_eq!(l, r $(, $opt = $val)*)),
            "assertion failed: `(left ≈ right)`\n\
                left: `{:?}`\n\
                right: `{:?}`",
            $lhs,
            $rhs
        );
    }};
}

#[macro_use]
pub mod integrators;
#[macro_use]
pub mod polynomials;

pub mod double_factorial;
pub mod evaluation;
pub mod orthogonal_polynomials;
pub mod spherical_harmonics;

pub use evaluation::{Evaluate, EvaluateBounded};

/// Map `val`, which has a value within `source_range`, to `target_range`.
pub fn normalize(
    source_range: RangeInclusive<f32>,
    target_range: RangeInclusive<f32>,
    val: f32,
) -> f32 {
    (val - source_range.start()) / (source_range.end() - source_range.start())
        * (target_range.end() - target_range.start())
        + target_range.start()
}

/// Apply [`normalize`] to an entire collection of values.
pub fn normalize_collection<'a>(
    source_range: RangeInclusive<f32>,
    target_range: RangeInclusive<f32>,
    vals: impl IntoIterator<Item = &'a mut f32>,
) {
    vals.into_iter()
        .for_each(|v| *v = normalize(source_range.clone(), target_range.clone(), *v));
}

/// Try to find roots of a continuous function in a given `interval`. `num_initial_test` evenly
/// spaced points are used to test for sign changes (thus, zeros, by the intermediate value
/// theorem), which are then refined using Brent's method.
///
/// # Panics
/// This function will panic if the root finder does not converge.
pub fn find_roots_in_interval<'a>(
    interval: RangeInclusive<f32>,
    num_initial_tests: usize,
    f: impl Fn(f32) -> f32 + Copy + 'a,
) -> impl Iterator<Item = f32> + 'a {
    interval
        .linspace(num_initial_tests)
        .map(move |a| (a, f(a)))
        .tuple_windows()
        .filter(|((_, f_a), (_, f_b))| f_a * f_b < 0.0) // ab < 0 iff a < 0 xor b < 0.
        .map(move |((a, _), (b, _))| {
            roots::find_root_brent(a, b, f, &mut 1E-4_f32)
                .unwrap_or_else(|err| panic!("root finder encountered an error: {err}"))
        })
}
