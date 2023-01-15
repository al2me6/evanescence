use std::ops::RangeInclusive;

use evanescence_core::geometry::storage::Soa;
use evanescence_core::numerics::statistics::Pdf;
use evanescence_core::numerics::Function;
use evanescence_core::orbital::atomic::{Radial, RadialProbabilityDistribution};
use evanescence_core::orbital::{AtomicComplex, AtomicReal, Qn};
use na::vector;

use super::{Computation, ExecuteComputation};
use crate::computation_host::ComputationHost;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RadialComputationKind {
    Wavefunction,
    ProbabilityDistribution,
    CumulativeDistribution,
}

pub struct RadialComputation {
    pub kind: RadialComputationKind,
    pub count: usize,
    pub interval: RangeInclusive<f32>,
}

impl Computation<AtomicReal> for RadialComputation {
    type Output<'a> = Soa<1, f32>;

    fn execute<'a>(&self, _host: &'a mut ComputationHost, params: Qn) -> Self::Output<'a> {
        let nl = params.into();
        let interval_vector = vector![*self.interval.start()]..=vector![*self.interval.end()];
        match self.kind {
            RadialComputationKind::Wavefunction => {
                Radial::new(nl).sample_from_line_segment(interval_vector, self.count)
            }
            RadialComputationKind::ProbabilityDistribution => {
                RadialProbabilityDistribution::new(nl)
                    .sample_from_line_segment(interval_vector, self.count)
            }
            RadialComputationKind::CumulativeDistribution => {
                Pdf::new(RadialProbabilityDistribution::new(nl))
                    .sample_cdf(self.interval.clone(), self.count)
            }
        }
    }
}

impl Computation<AtomicComplex> for RadialComputation {
    type Output<'a> = Soa<1, f32>;

    fn execute<'a>(&self, host: &'a mut ComputationHost, params: Qn) -> Self::Output<'a> {
        self.execute_with::<AtomicReal>(host, params)
    }
}
