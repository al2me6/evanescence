pub mod point_cloud;
pub mod radial;

use crate::computation_host::ComputationHost;
use crate::evaluator::Evaluator;

pub trait Computation<E: Evaluator> {
    type Output<'a>: Clone;

    fn execute<'a>(&self, host: &'a mut ComputationHost, params: E::Params) -> Self::Output<'a>;
}

pub trait ExecuteComputation {
    fn execute_with<'a, E: Evaluator>(
        &self,
        host: &'a mut ComputationHost,
        params: E::Params,
    ) -> Self::Output<'a>
    where
        Self: Computation<E>,
    {
        self.execute(host, params)
    }
}

impl<C> ExecuteComputation for C {}
