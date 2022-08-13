use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use evanescence_core::geometry::PointValue;
use evanescence_core::numerics::monte_carlo::accept_reject::AcceptReject;
use evanescence_core::numerics::monte_carlo::MonteCarlo;
use evanescence_core::orbital::{self, Qn};
use num::complex::Complex32;

use super::MonteCarloParameters;

type DynMonteCarlo<O> = dyn MonteCarlo<Output = O> + Send + Sync;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum CacheKey {
    Real(Qn),
    Complex(Qn),
    /// Note that `Kind` is not and cannot be `Hash`. This should:tm: be sufficient.
    Hybrid {
        n: u32,
        symmetry: String,
        description: Option<String>,
    },
}

impl From<&MonteCarloParameters> for CacheKey {
    fn from(params: &MonteCarloParameters) -> Self {
        match params {
            MonteCarloParameters::AtomicReal(qn) => Self::Real(*qn),
            MonteCarloParameters::AtomicComplex(qn) => Self::Complex(*qn),
            MonteCarloParameters::Hybrid(kind) => Self::Hybrid {
                n: kind.n(),
                symmetry: kind.symmetry().clone(),
                description: kind.description().clone(),
            },
        }
    }
}

struct CacheEntry<O: Copy> {
    sampler: Box<DynMonteCarlo<O>>,
    samples: Vec<PointValue<O>>,
}

impl<O: Copy> CacheEntry<O> {
    fn new(sampler: Box<DynMonteCarlo<O>>) -> Self {
        Self {
            sampler,
            samples: Vec::with_capacity(1 >> 14),
        }
    }

    fn request_simulation(&mut self, count: usize) -> impl Iterator<Item = &PointValue<O>> {
        if self.samples.len() < count {
            let count = count - self.samples.len();
            log::debug!("[MonteCarlo cache] simulating {count} samples.");
            self.samples.extend(self.sampler.simulate(count));
        }
        self.samples[..count].iter()
    }
}

#[derive(Default)]
pub struct MonteCarloCache {
    cache_real: HashMap<CacheKey, CacheEntry<f32>>,
    cache_complex: HashMap<CacheKey, CacheEntry<Complex32>>,
}

impl MonteCarloCache {
    pub fn request_f32(
        &mut self,
        params: MonteCarloParameters,
        count: usize,
    ) -> Option<impl Iterator<Item = &PointValue<f32>>> {
        if matches!(params, MonteCarloParameters::AtomicComplex(_)) {
            return None;
        }
        let entry = self
            .cache_real
            .entry(CacheKey::from(&params))
            .or_insert_with(|| {
                CacheEntry::new(match params {
                    MonteCarloParameters::AtomicReal(qn) => {
                        Box::new(AcceptReject::new(orbital::Real::new(qn)))
                    }
                    MonteCarloParameters::Hybrid(kind) => Box::new(AcceptReject::new(
                        orbital::hybrid::Hybrid::new(kind.archetype().clone()),
                    )),
                    MonteCarloParameters::AtomicComplex(_) => unreachable!(),
                })
            });
        Some(entry.request_simulation(count))
    }

    pub fn request_complex32(
        &mut self,
        params: MonteCarloParameters,
        count: usize,
    ) -> Option<impl Iterator<Item = &PointValue<Complex32>>> {
        if !matches!(params, MonteCarloParameters::AtomicComplex(_)) {
            return None;
        }
        let entry = self
            .cache_complex
            .entry(CacheKey::from(&params))
            .or_insert_with(|| match params {
                MonteCarloParameters::AtomicComplex(qn) => {
                    CacheEntry::new(Box::new(AcceptReject::new(orbital::Complex::new(qn))))
                }
                _ => unreachable!(),
            });
        Some(entry.request_simulation(count))
    }
}

pub static MONTE_CARLO_CACHE: LazyLock<Mutex<MonteCarloCache>> =
    LazyLock::new(|| Mutex::new(Default::default()));
