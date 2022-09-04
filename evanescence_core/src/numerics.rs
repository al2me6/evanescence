#[macro_use]
pub mod integrators;
#[macro_use]
pub mod polynomial;

pub mod function;
pub mod monte_carlo;
pub mod optimization;
pub mod random;
pub mod root_finding;
pub mod special;
pub mod statistics;

/// Additional `f32` constants.
pub mod consts {
    pub const FRAC_1_SQRT_3: f32 = 0.577_350_3;
    pub const FRAC_1_SQRT_6: f32 = 0.408_248_3;
    pub const SQRT_3: f32 = 1.732_050_8;
}

use std::ops::{AddAssign, Div, Neg, RangeInclusive, Sub};

use na::Vector3;

pub use self::function::Function;

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

/// Perform trilinear interpolation given the values of a function at the vertices of a rectangular
/// prism, passed as a `[f32; 8]` in iteration order `xyz`. (_e.g._, the first element is the
/// lower left corner and the last is the top right.)
///
/// `normalized_offset` is the normalized position of the interpolation target, _i.e._ the vector
/// `pt - bottom_left`, divided coordinate-wise by the lattice size in the corresponding dimension.
#[allow(clippy::similar_names)]
pub fn trilinear_interpolate(
    [f000, f001, f010, f011, f100, f101, f110, f111]: [f32; 8],
    normalized_offset: Vector3<f32>,
) -> f32 {
    let d = normalized_offset;
    let f00 = f000 * (1. - d.x) + f100 * d.x;
    let f01 = f001 * (1. - d.x) + f101 * d.x;
    let f10 = f010 * (1. - d.x) + f110 * d.x;
    let f11 = f011 * (1. - d.x) + f111 * d.x;
    let f0 = f00 * (1. - d.y) + f10 * d.y;
    let f1 = f01 * (1. - d.y) + f11 * d.y;
    f0 * (1. - d.z) + f1 * d.z
}
