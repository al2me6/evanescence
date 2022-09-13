//! Implementation of hybrid orbitals.

#[macro_use]
pub mod linear_combination;
#[macro_use]
pub mod kind;

pub mod library;

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
        Iterator::zip(self.lc.iter(), &self.reals)
            .map(|(Component { weight, .. }, real)| weight * real.evaluate(point))
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
        let PointValue(_, max) =
            Simple::new(self.bounding_simplex(), |pt| self.evaluate(pt).abs(), 0.15)
                .maximize(75_000);

        self.probability_density_of(max)
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use super::{library, Hybrid};
    use crate::geometry::point::{IPoint, SphericalPoint3};
    use crate::geometry::region::{BoundingRegion, Region};
    use crate::numerics::monte_carlo::accept_reject::AcceptRejectParameters;
    use crate::numerics::random::WyRand;
    use crate::numerics::statistics::Distribution;

    #[test]
    fn max_prob_density_computation() {
        const BRUTE_FORCE_SAMPLE_COUNT: usize = 2_000_000;

        for kind in [
            &library::SP,
            &library::SP2,
            &library::SP3,
            &library::SP3D, // Its archetype does not have s character and is hard to optimize.
            &library::SP3D2,
        ] {
            let orbital = Hybrid::new(kind.archetype().clone());

            let rng = &mut WyRand::new();
            let region = orbital.bounding_region();
            let brute_force_max = Iterator::chain(
                iter::once(SphericalPoint3::origin()),
                iter::repeat_with(|| region.sample(rng)),
            )
            .take(BRUTE_FORCE_SAMPLE_COUNT)
            .map(|pt| orbital.probability_density(&pt))
            .reduce(f32::max)
            .unwrap();

            let optimization_max = <_ as AcceptRejectParameters<3, _>>::maximum(&orbital);

            println!(
                "{} brute-force: {brute_force_max}; optimization: {optimization_max}",
                **kind
            );
            assert!(brute_force_max * (1.0 - 5E-3) <= optimization_max);
        }
    }
}
