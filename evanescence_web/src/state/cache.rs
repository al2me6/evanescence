use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use evanescence_core::geometry::storage::{PointValue, Soa, SoaSlice};
use evanescence_core::numerics::monte_carlo::accept_reject::AcceptReject;
use evanescence_core::orbital::hybrid::Hybrid;
use evanescence_core::orbital::{AtomicComplex, AtomicReal, Qn};
use na::SVector;
use num::complex::Complex32;

use super::MonteCarloParameters;

type DynMonteCarlo<V> = dyn Iterator<Item = (SVector<f32, 3>, V)> + Send + Sync;

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

struct CacheEntry<V: Copy> {
    sampler: Box<DynMonteCarlo<V>>,
    samples: Soa<3, V>,
}

macro_rules! cache_entry {
    ($evaluator:expr) => {
        CacheEntry::new(Box::new(
            AcceptReject::new($evaluator).map(PointValue::into_raw),
        ))
    };
}

impl<V: Copy> CacheEntry<V> {
    fn new(sampler: Box<DynMonteCarlo<V>>) -> Self {
        Self {
            sampler,
            samples: Soa::with_capacity(1 >> 14),
        }
    }

    fn request_simulation(&mut self, count: usize) -> SoaSlice<'_, 3, V> {
        if self.samples.len() < count {
            let count = count - self.samples.len();
            log::debug!("[MonteCarlo cache] simulating {count} samples");
            self.samples.extend(self.sampler.by_ref().take(count));
        }
        self.samples.slice(..count)
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
    ) -> Option<SoaSlice<'_, 3, f32>> {
        if matches!(params, MonteCarloParameters::AtomicComplex(_)) {
            return None;
        }
        let entry = self
            .cache_real
            .entry(CacheKey::from(&params))
            .or_insert_with(|| match params {
                MonteCarloParameters::AtomicReal(qn) => cache_entry!(AtomicReal::new(qn)),
                MonteCarloParameters::Hybrid(kind) => {
                    cache_entry!(Hybrid::new(kind.archetype().clone()))
                }
                MonteCarloParameters::AtomicComplex(_) => unreachable!(),
            });
        Some(entry.request_simulation(count))
    }

    pub fn request_complex32(
        &mut self,
        params: MonteCarloParameters,
        count: usize,
    ) -> Option<SoaSlice<'_, 3, Complex32>> {
        if !matches!(params, MonteCarloParameters::AtomicComplex(_)) {
            return None;
        }
        let entry = self
            .cache_complex
            .entry(CacheKey::from(&params))
            .or_insert_with(|| match params {
                MonteCarloParameters::AtomicComplex(qn) => cache_entry!(AtomicComplex::new(qn)),
                _ => unreachable!(),
            });
        Some(entry.request_simulation(count))
    }
}

pub static MONTE_CARLO_CACHE: LazyLock<Mutex<MonteCarloCache>> =
    LazyLock::new(|| Mutex::new(Default::default()));
