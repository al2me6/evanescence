use std::ops::{AddAssign, Div, Neg, RangeInclusive, Sub};

/// Verify that two iterables containing float values are approximately equal.
#[cfg(test)]
macro_rules! assert_iterable_approx_eq {
    ($lhs:expr, $rhs: expr $(, $opt:ident = $val:expr)* $(,)?) => {
        assert_iterable_approx_eq!(relative_eq, $lhs, $rhs $(, $opt = $val)*)
    };
    ($method:ident, $lhs:expr, $rhs: expr $(, $opt:ident = $val:expr)* $(,)?) => {{
        use itertools::Itertools;
        assert!(
            $lhs.iter()
                .zip_eq($rhs.iter())
                .all(|(l, r)| approx::$method!(l, r $(, $opt = $val)*)),
            "assertion failed: `(left ≈ right)` via {}\n\
                left: `{:?}`\n\
                right: `{:?}`",
            stringify!($method),
            $lhs,
            $rhs
        );
    }};
}

#[macro_use]
pub mod integrators;
#[macro_use]
pub mod polynomial;

pub mod function;
pub mod monte_carlo;
pub mod random;
pub mod root_finding;
pub mod special;
pub mod statistics;

pub use function::Function;

/// Produce `num_points` values evenly spaced across `interval`.
pub fn linspace<T>(
    interval: RangeInclusive<T>,
    num_points: usize,
) -> impl ExactSizeIterator<Item = T> + Clone
where
    for<'a> T: AddAssign<&'a T> + Sub<&'a T, Output = T> + Div<f32, Output = T> + Clone,
{
    let step = (interval.end().clone() - interval.start()) / (num_points as f32 - 1.0);
    let mut acc = interval.start().clone();
    (0..num_points).map(move |_| {
        let next = acc.clone();
        acc += &step;
        next
    })
}

/// Produce `num_points` values evenly spaced across `[-range, range]`.
pub fn symmetric_linspace<T>(
    range: T,
    num_points: usize,
) -> impl ExactSizeIterator<Item = T> + Clone
where
    for<'a> T:
        AddAssign<&'a T> + Sub<&'a T, Output = T> + Div<f32, Output = T> + Neg<Output = T> + Clone,
{
    linspace((-range.clone())..=range, num_points)
}

/// Map `val`, which has a value within `source_range`, to `target_range`.
pub fn normalize(
    source_range: RangeInclusive<f32>,
    target_range: RangeInclusive<f32>,
    val: f32,
) -> f32 {
    let (source_start, source_end) = source_range.into_inner();
    let (target_start, target_end) = target_range.into_inner();
    (val - source_start) / (source_end - source_start) * (target_end - target_start) + target_start
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
