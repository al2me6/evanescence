use num::complex::Complex32;

use super::Radial;
use crate::geometry::point::{SphericalCoordinatesExt, SphericalPoint3};
use crate::geometry::region::{BallCenteredAtOrigin, BoundingRegion};
use crate::numerics::monte_carlo::accept_reject::AcceptRejectParameters;
use crate::numerics::special::spherical_harmonics::SphericalHarmonic;
use crate::numerics::statistics::Distribution;
use crate::numerics::Function;
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

impl Function<3, SphericalPoint3> for Complex {
    type Output = Complex32;

    #[inline]
    fn evaluate(&self, point: &SphericalPoint3) -> Complex32 {
        self.radial.evaluate(&[point.r()].into()) * self.sph.evaluate(point)
    }
}

impl BoundingRegion<3, SphericalPoint3> for Complex {
    type Geometry = BallCenteredAtOrigin;

    fn bounding_region(&self) -> Self::Geometry {
        BallCenteredAtOrigin {
            radius: super::bound(self.qn),
        }
    }
}

impl Distribution<3, SphericalPoint3> for Complex {
    #[inline]
    fn probability_density_of(&self, value: Self::Output) -> f32 {
        let norm = value.norm();
        norm * norm
    }
}

impl Orbital<SphericalPoint3> for Complex {
    /// Give the name of the wavefunction (ex. `ψ_{420}`).
    fn name(&self) -> String {
        Self::name_qn(self.qn)
    }
}

impl AcceptRejectParameters<3, SphericalPoint3> for Complex {
    // TODO: custom maximum impl.

    fn accept_threshold_fudge(&self) -> Option<f32> {
        Some(super::accept_threshold_modifier(self.qn))
    }
}
