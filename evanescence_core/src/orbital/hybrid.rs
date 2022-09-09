//! Implementation of hybrid orbitals.

#[macro_use]
pub mod linear_combination;
#[macro_use]
pub mod kind;

pub use self::kind::Kind;
use self::linear_combination::Component;
pub use self::linear_combination::LinearCombination;
use super::{AtomicReal, Orbital};
use crate::geometry::point::SphericalPoint3;
use crate::geometry::region::{BallCenteredAtOrigin, BoundingRegion};
use crate::geometry::storage::PointValue;
use crate::numerics::monte_carlo::accept_reject::AcceptRejectParameters;
use crate::numerics::optimization::simple_x::{BoundingSimplex, Simple};
use crate::numerics::statistics::Distribution;
use crate::numerics::Function;
use crate::utils::sup_sub_string::SupSubString;

/// Implementation of hybrid orbitals.
#[derive(Clone, PartialEq, Debug)]
pub struct Hybrid {
    lc: LinearCombination,
    reals: Vec<AtomicReal>,
}

impl Hybrid {
    pub fn new(lc: LinearCombination) -> Self {
        let reals = lc.iter().map(|comp| AtomicReal::new(comp.qn)).collect();
        Self { lc, reals }
    }
}

impl Function<3, SphericalPoint3> for Hybrid {
    type Output = f32;

    #[inline]
    fn evaluate(&self, point: &SphericalPoint3) -> Self::Output {
        self.lc
            .iter()
            .enumerate()
            .map(|(idx, Component { weight, .. })| weight * self.reals[idx].evaluate(point))
            .sum()
    }
}

impl BoundingRegion<3, SphericalPoint3> for Hybrid {
    type Geometry = BallCenteredAtOrigin;

    fn bounding_region(&self) -> Self::Geometry {
        BallCenteredAtOrigin {
            radius: self
                .reals
                .iter()
                .map(|real| real.bounding_region().radius)
                .reduce(f32::max)
                .expect("linear combination must contain at least one orbital")
                * 0.9,
        }
    }
}

impl Distribution<3, SphericalPoint3> for Hybrid {
    #[inline]
    fn probability_density_of(&self, value: Self::Output) -> f32 {
        value * value
    }
}

impl Orbital<SphericalPoint3> for Hybrid {
    fn name(&self) -> SupSubString {
        self.lc.expression()
    }
}

impl AcceptRejectParameters<3, SphericalPoint3> for Hybrid {
    fn maximum(&self) -> f32 {
        const ITERS: usize = 400;
        const EXPLORATION_PREFERENCE: f32 = 0.3;

        let PointValue(_, max) = Simple::new(
            self.bounding_simplex(),
            |pt| self.evaluate(pt).abs(),
            EXPLORATION_PREFERENCE,
        )
        .maximize(ITERS);

        self.probability_density_of(max)
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_1_SQRT_2;
    use std::iter;

    use super::Hybrid;
    use crate::geometry::point::{IPoint, SphericalPoint3};
    use crate::geometry::region::{BoundingRegion, Region};
    use crate::numerics::consts::{FRAC_1_SQRT_6, SQRT_3};
    use crate::numerics::monte_carlo::accept_reject::AcceptRejectParameters;
    use crate::numerics::random::WyRand;
    use crate::numerics::statistics::Distribution;

    #[test]
    fn max_prob_density_computation() {
        const BRUTE_FORCE_SAMPLE_COUNT: usize = 2_000_000;

        let sp3d2 = Hybrid::new(lc! {
            overall: FRAC_1_SQRT_6,
            (3, 0, 0) * 1.0,
            (3, 1, 1) * SQRT_3,
            (3, 2, 0) * -FRAC_1_SQRT_2,
            (3, 2, 2) * SQRT_3 * FRAC_1_SQRT_2,
        });

        let rng = &mut WyRand::new();
        let region = sp3d2.bounding_region();
        let brute_force_max = Iterator::chain(
            iter::once(SphericalPoint3::origin()),
            iter::repeat_with(|| region.sample(rng)),
        )
        .take(BRUTE_FORCE_SAMPLE_COUNT)
        .map(|pt| sp3d2.probability_density(&pt))
        .reduce(f32::max)
        .unwrap();

        let explicit_max = <_ as AcceptRejectParameters<3, _>>::maximum(&sp3d2);

        println!("brute-force: {brute_force_max}; explicit: {explicit_max}");
        assert!(brute_force_max * (1.0 - 1E-6) <= explicit_max);
    }
}
