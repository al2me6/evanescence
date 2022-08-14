use std::ops::RangeInclusive;

use itertools::Itertools;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RootError {
    #[error("failed to converge to the desired tolerance within the step limit")]
    FailedToConverge,
    #[error("the passed interval does not bracket a root")]
    NotBracketed,
}

/// Give the root of `f` that is bracketed within `interval`.
///
/// # Errors
/// Errors if the `interval` does not actually contain a root, or if the algorithm fails to
/// converge to `tolerance` within `max_iters` iterations (unlimited if `None`).
#[allow(clippy::many_single_char_names)] // Convention.
pub fn brent(
    interval: RangeInclusive<f32>,
    mut f: impl FnMut(f32) -> f32,
    tolerance: f32,
    max_iters: Option<usize>,
) -> Result<f32, RootError> {
    fn same_sign_and_nonzero(a: f32, b: f32) -> bool {
        a > 0. && b > 0. || a < 0. && b < 0.
    }

    let (mut a, mut b) = interval.into_inner();
    let mut c = b;
    let mut d = 0.;
    let mut e = 0.;
    let mut f_a = f(a);
    let mut f_b = f(b);
    let mut f_c = f_b;
    if same_sign_and_nonzero(f_a, f_b) {
        Err(RootError::NotBracketed)?;
    }
    for _ in 0..max_iters.unwrap_or(usize::MAX) {
        if same_sign_and_nonzero(f_b, f_c) {
            (c, f_c) = (a, f_a);
            d = b - a;
            e = d;
        }
        if f_c.abs() < f_b.abs() {
            (a, b, c) = (b, c, b);
            (f_a, f_b, f_c) = (f_b, f_c, f_b);
        }
        let delta = 2. * f32::EPSILON * b.abs() + 0.5 * tolerance;
        let m = 0.5 * (c - b);
        if m.abs() <= delta || f_b == 0. {
            return Ok(b);
        }
        if e.abs() < delta || f_a.abs() <= f_b.abs() {
            // Bisection.
            d = m;
            e = m;
        } else {
            let (mut p, mut q);
            #[allow(clippy::float_cmp)]
            if a == c {
                // Linear interpolation.
                let s = f_b / f_a;
                p = 2. * m - s;
                q = 1. - s;
            } else {
                // Inverse quadratic interpolation.
                let r1 = f_a / f_c;
                let r2 = f_b / f_c;
                let r3 = f_b / f_a;
                p = r3 * (2. * m * r1 + (r1 - r2) - (b - a) * (r2 - 1.));
                q = (r1 - 1.) * (r2 - 1.) * (r3 - 1.);
            }
            if p > 0. {
                q = -q;
            } else {
                p = -p;
            }
            if 2. * p < 3. * m * q - (delta * q).abs() && p < (0.5 * e * q).abs() {
                // Accept interpolation.
                e = d;
                d = p / q;
            } else {
                // Reject; use bisection.
                d = m;
                e = d;
            }
        }
        (a, f_a) = (b, f_b);
        b += if d.abs() > m { d } else { delta.copysign(m) };
        f_b = f(b);
    }
    Err(RootError::FailedToConverge)
}

/// Try to find roots of a continuous function in a given `interval`. `num_initial_test` evenly
/// spaced points are used to test for sign changes (thus, zeros, by the intermediate value
/// theorem), which are then refined using Brent's method.
///
/// # Errors
/// This function will return an error if the root finder does not converge.
#[allow(clippy::missing_panics_doc)] // Sanity check.
pub fn find_roots_in_interval_brent(
    interval: RangeInclusive<f32>,
    num_initial_tests: usize,
    f: impl Fn(f32) -> f32 + Copy,
) -> Result<Vec<f32>, RootError> {
    super::linspace(interval, num_initial_tests)
        .map(|a| (a, f(a)))
        .tuple_windows()
        .filter(|((_, f_a), (_, f_b))| f_a * f_b < 0.0) // ab < 0 iff a < 0 xor b < 0.
        .map(|((a, _), (b, _))| brent(a..=b, f, 1E-6, None))
        .inspect(|res| assert!(!matches!(res, &Err(RootError::NotBracketed))))
        .collect()
}
