use num::complex::Complex32;

use super::Radial;
use crate::geometry::region::{BallCenteredAtOrigin, BoundingRegion};
use crate::geometry::SphericalPoint3;
use crate::numerics::monte_carlo::accept_reject::AcceptRejectParameters;
use crate::numerics::spherical_harmonics::SphericalHarmonic;
use crate::numerics::statistics::Distribution;
use crate::numerics::Evaluate;
use crate::orbital::{Orbital, Qn};

/// Implementation of the complex hydrogenic orbitals.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Complex {
    qn: Qn,
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
    fn evaluate(&self, point: &SphericalPoint3) -> Complex32 {
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

impl Orbital for Complex {
    /// Give the name of the wavefunction (ex. `ψ_{420}`).
    fn name(&self) -> String {
        Self::name_qn(self.qn)
    }
}

impl AcceptRejectParameters for Complex {
    // TODO: custom maximum impl.

    fn accept_threshold_fudge(&self) -> Option<f32> {
        Some(super::accept_threshold_modifier(self.qn))
    }
}
