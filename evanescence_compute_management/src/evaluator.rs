use evanescence_core::geometry::storage::PointValue;
use evanescence_core::numerics::monte_carlo::accept_reject::AcceptReject;
use evanescence_core::orbital::{self, AtomicComplex, AtomicReal, Qn};
use na::SVector;
use num::complex::Complex32;
use num::Num;

#[derive(Clone, Copy, Debug, derivative::Derivative)]
#[derivative(PartialEq, Eq, Hash)]
pub enum ParametersEnum {
    Atomic(Qn),
    Hybrid {
        #[derivative(PartialEq(compare_with = "std::ptr::eq"))]
        #[derivative(Hash(hash_with = "std::ptr::hash"))]
        kind: &'static orbital::hybrid::Kind,
    },
}

impl From<Qn> for ParametersEnum {
    fn from(qn: Qn) -> Self {
        Self::Atomic(qn)
    }
}

pub(crate) type MonteCarloSampler<S> = Box<dyn Iterator<Item = (SVector<f32, 3>, S)>>;

pub trait Evaluator: Clone + 'static {
    type Scalar: Num + Copy + 'static;
    type Params: Copy + Into<ParametersEnum> + 'static;
    fn make(params: Self::Params) -> Self
    where
        Self: Sized;
    fn make_monte_carlo_sampler(&self) -> MonteCarloSampler<Self::Scalar>;
}

impl Evaluator for AtomicReal {
    type Params = Qn;
    type Scalar = f32;

    fn make(params: Self::Params) -> Self {
        Self::new(params)
    }

    fn make_monte_carlo_sampler(&self) -> MonteCarloSampler<Self::Scalar> {
        Box::new(AcceptReject::new(self.clone()).map(PointValue::into_raw))
    }
}

impl Evaluator for AtomicComplex {
    type Params = Qn;
    type Scalar = Complex32;

    fn make(params: Self::Params) -> Self {
        Self::new(params)
    }

    fn make_monte_carlo_sampler(&self) -> MonteCarloSampler<Self::Scalar> {
        Box::new(AcceptReject::new(self.clone()).map(PointValue::into_raw))
    }
}
