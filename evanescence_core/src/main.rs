use evanescence_core::monte_carlo::{MonteCarlo, Quality};
use evanescence_core::orbital::{QuantumNumbers, RealOrbital};
fn main() {
    let result =
        RealOrbital::monte_carlo_simulate(QuantumNumbers::new(4, 2, 0).unwrap(), Quality::VeryHigh);
    for i in 0..result.xs.len() {
        println!(
            "({}, {}, {}), {}",
            result.xs[i], result.ys[i], result.zs[i], result.vals[i]
        );
    }
}
