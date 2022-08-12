use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use evanescence_core::geometry::PointValue;
use evanescence_core::numerics::monte_carlo::accept_reject::AcceptReject;
use evanescence_core::numerics::monte_carlo::MonteCarlo;
use evanescence_core::orbital::{self, Qn};
use num::complex::Complex32;

use super::{Mode, State};

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

impl From<&State> for CacheKey {
    fn from(state: &State) -> Self {
        match state.mode() {
            Mode::RealSimple | Mode::RealFull => Self::Real(*state.qn()),
            Mode::Hybrid => {
                let kind = state.hybrid_kind();
                Self::Hybrid {
                    n: kind.n(),
                    symmetry: kind.symmetry().clone(),
                    description: kind.description().clone(),
                }
            }
            Mode::Complex => Self::Complex(*state.qn()),
        }
    }
}

struct CacheEntry<O: Copy> {
    evaluator: Box<DynMonteCarlo<O>>,
    samples: Vec<PointValue<O>>,
}

impl<O: Copy> CacheEntry<O> {
    fn new(evaluator: Box<DynMonteCarlo<O>>) -> Self {
        Self {
            evaluator,
            samples: Vec::with_capacity(1 >> 14),
        }
    }

    fn request_simulation(&mut self, count: usize) -> impl Iterator<Item = &PointValue<O>> {
        if self.samples.len() < count {
            self.samples
                .extend(self.evaluator.simulate(count - self.samples.len()));
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
    pub fn request_f32(&mut self, state: &State) -> Option<impl Iterator<Item = &PointValue<f32>>> {
        if state.mode().is_complex() {
            return None;
        }
        let entry = self
            .cache_real
            .entry(CacheKey::from(state))
            .or_insert_with(|| {
                CacheEntry::new(match state.mode() {
                    Mode::RealSimple | Mode::RealFull => {
                        Box::new(AcceptReject::new(orbital::Real::new(*state.qn())))
                    }
                    Mode::Hybrid => Box::new(AcceptReject::new(orbital::hybrid::Hybrid::new(
                        state.hybrid_kind().archetype().clone(),
                    ))),
                    Mode::Complex => unreachable!(),
                })
            });
        Some(entry.request_simulation(state.quality().point_cloud()))
    }

    pub fn request_complex32(
        &mut self,
        state: &State,
    ) -> Option<impl Iterator<Item = &PointValue<Complex32>>> {
        if !state.mode().is_complex() {
            return None;
        }
        let entry = self
            .cache_complex
            .entry(CacheKey::from(state))
            .or_insert_with(|| match state.mode() {
                Mode::Complex => CacheEntry::new(Box::new(AcceptReject::new(
                    orbital::Complex::new(*state.qn()),
                ))),
                _ => unreachable!(),
            });
        Some(entry.request_simulation(state.quality().point_cloud()))
    }
}

pub static MONTE_CARLO_CACHE: LazyLock<Mutex<MonteCarloCache>> =
    LazyLock::new(|| Mutex::new(Default::default()));
