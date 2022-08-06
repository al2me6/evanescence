use num::complex::Complex32;

use super::Radial;
use crate::geometry::region::{BallCenteredAtOrigin, BoundingRegion};
use crate::geometry::Point;
use crate::numerics::monte_carlo::accept_reject::{AcceptRejectFudge, MaximumInBoundingRegion};
use crate::numerics::spherical_harmonics::SphericalHarmonic;
use crate::numerics::statistics::Distribution;
use crate::numerics::Evaluate;
use crate::orbital::{Orbital, Qn};

/// Implementation of the complex hydrogenic orbitals.
pub struct Complex {
    pub(in crate::orbital) qn: Qn,
    radial: Radial,
    sph: SphericalHarmonic,
}

impl Complex {
    pub fn new(qn: Qn) -> Self {
        Self {
            qn,
            radial: Radial::new(qn.into()),
            sph: SphericalHarmonic::new(qn.into()),
        }
    }

    pub fn name_qn(qn: Qn) -> String {
        super::basic_name(qn)
    }
}

impl Evaluate for Complex {
    type Output = Complex32;

    #[inline]
    fn evaluate(&self, point: &Point) -> Complex32 {
        self.radial.evaluate(point) * self.sph.evaluate(point)
    }
}

impl BoundingRegion for Complex {
    type Geometry = BallCenteredAtOrigin;

    fn bounding_region(&self) -> Self::Geometry {
        BallCenteredAtOrigin {
            radius: super::bound(self.qn),
        }
    }
}

impl Distribution for Complex {
    #[inline]
    fn probability_density_of(&self, value: Self::Output) -> f32 {
        let norm = value.norm();
        norm * norm
    }
}

impl MaximumInBoundingRegion for Complex {
    // TODO: custom impl.
}

impl Orbital for Complex {
    /// Give the name of the wavefunction (ex. `ψ_{420}`).
    fn name(&self) -> String {
        Self::name_qn(self.qn)
    }
}

impl AcceptRejectFudge for Complex {
    fn accept_threshold_modifier(&self) -> Option<f32> {
        Some(super::accept_threshold_modifier(self.qn))
    }
}
