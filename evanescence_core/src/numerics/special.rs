pub mod factorial;
pub mod orthogonal_polynomials;
pub mod spherical_harmonics;

/// See <https://stackoverflow.com/a/52795863>
///
/// # Panics
/// Panics if `k > n`.
pub fn binomial_coefficient(n: u32, k: u32) -> u32 {
    fn tail(max_k: u32, n: u32, k: u32, acc: u32) -> u32 {
        if k > max_k {
            return acc;
        }
        #[allow(clippy::integer_division)]
        tail(max_k, n + 1, k + 1, (n * acc) / k)
    }
    assert!(n >= k);
    if k == 0 || k == n {
        1
    } else {
        tail(k, n - k + 1, 1, 1)
    }
}
