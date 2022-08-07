use std::ops::RangeInclusive;

use crate::numerics::normalize;

/// The Kolmogorov-Smirnov distribution.
///
/// Ref. _Numerical Recipes_ 3rd ed., section 6.14.12.
///
/// # Panics
/// Panics if `z < 0`.
pub fn kolmogorov_smirnov_p(z: f32) -> f32 {
    assert!(z >= 0.);
    if z == 0. {
        0.
    } else if z < 1.18 {
        let y = (-1.233_700_6 / (z * z)).exp();
        2.256_758_3 * (-y.ln()).sqrt() * (y + y.powi(9) + y.powi(25) + y.powi(49))
    } else {
        let x = (-2. * z * z).exp();
        1. - 2. * (x - x.powi(4) + x.powi(9))
    }
}

/// The complement of the Kolmogorov-Smirnov distribution.
/// # Panics
/// Panics if `z < 0`.
pub fn kolmogorov_smirnov_q(z: f32) -> f32 {
    1. - kolmogorov_smirnov_p(z)
}

/// Perform the Kolmogorov-Smirnov test given a sorted list of samples and the CDF of the
/// expected distribution.
///
/// Ref. _Numerical Recipes_ 3rd ed., section 14.3.3.
///
/// # Panics
/// Panics if `data` is not sorted, `data` is empty, or `cdf` returns a value not in `[0, 1]`.
pub fn kolmogorov_smirnov_test(data: &[f32], mut cdf: impl FnMut(f32) -> f32) -> (f32, f32) {
    assert!(data.is_sorted());

    let min_expected_cdf = cdf(data[0]);
    let max_expected_cdf = cdf(*data.last().expect("data must be nonempty"));
    assert!(min_expected_cdf >= 0. && max_expected_cdf <= 1.);

    let mut data_cdf_prev = min_expected_cdf;
    let mut ks_statistic = 0.;

    for (i, sample) in data.iter().enumerate() {
        let data_cdf = normalize(
            0.0..=1.0,
            min_expected_cdf..=max_expected_cdf,
            (i as f32 + 1.) / data.len() as f32,
        );
        let expected_cdf = cdf(*sample);
        assert!((0.0..=1.0).contains(&expected_cdf));

        let delta = f32::max(
            (data_cdf_prev - expected_cdf).abs(),
            (data_cdf - expected_cdf).abs(),
        );
        ks_statistic = f32::max(ks_statistic, delta);
        data_cdf_prev = data_cdf;
    }

    let sqrt_n_e = (data.len() as f32).sqrt();
    let p = kolmogorov_smirnov_q((sqrt_n_e + 0.12 + 0.11 / sqrt_n_e) * ks_statistic);

    (ks_statistic, p)
}

/// Test whether `data` is uniformly distributed within `interval`.
pub fn test_uniformly_distributed_on(data: &[f32], interval: RangeInclusive<f32>) -> (f32, f32) {
    let cdf = |x| normalize(interval.clone(), 0.0..=1.0, x);
    kolmogorov_smirnov_test(data, cdf)
}
