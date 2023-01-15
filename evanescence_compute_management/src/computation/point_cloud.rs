use evanescence_core::geometry::storage::{Soa, SoaSlice};

use super::Computation;
use crate::computation_host::ComputationHost;
use crate::evaluator::{Evaluator, MonteCarloSampler};

pub struct PointCloudComputation {
    pub count: usize,
}

pub(crate) struct PointCloudComputationCache<S> {
    pub(crate) evaluator: MonteCarloSampler<S>,
    pub(crate) samples: Soa<3, S>,
}

impl<E: Evaluator> Computation<E> for PointCloudComputation {
    type Output<'a> = SoaSlice<'a, 3, E::Scalar>;

    fn execute<'a>(&self, host: &'a mut ComputationHost, params: E::Params) -> Self::Output<'a> {
        let PointCloudComputationCache { evaluator, samples } = host
            .get_or_insert_cache_entry::<Self, _>(params.into(), || PointCloudComputationCache {
                evaluator: E::make(params).make_monte_carlo_sampler(),
                samples: Soa::new(),
            });
        if samples.len() < self.count {
            samples.extend(evaluator.take(self.count - samples.len()));
        }
        samples.slice(..self.count)
    }
}
