use num::complex::Complex32;

use super::Radial;
use crate::geometry::Point;
use crate::numerics::spherical_harmonics::SphericalHarmonic;
use crate::numerics::{Evaluate, EvaluateBounded};
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

impl EvaluateBounded for Complex {
    #[inline]
    fn bound(&self) -> f32 {
        super::bound(self.qn)
    }
}

impl Orbital for Complex {
    #[inline]
    fn probability_density_of(&self, value: Self::Output) -> f32 {
        let norm = value.norm();
        norm * norm
    }

    /// Give the name of the wavefunction (ex. `ψ_{420}`).
    fn name(&self) -> String {
        Self::name_qn(self.qn)
    }
}
