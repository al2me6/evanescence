use evanescence_core::orbital::monte_carlo::{MonteCarlo, Quality};
use evanescence_core::orbital::{QuantumNumbers, RealOrbital};
fn main() {
    RealOrbital::monte_carlo_simulate(QuantumNumbers::new(4, 2, 0).unwrap(), Quality::VeryHigh)
        .iter()
        .for_each(|(pt, val)| {
            println!("{}, {}", pt, val);
        });
}
