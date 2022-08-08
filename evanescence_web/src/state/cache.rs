use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use evanescence_core::numerics::monte_carlo::accept_reject::AcceptReject;
use evanescence_core::numerics::monte_carlo::MonteCarlo;
use evanescence_core::orbital::{self, Qn};
use num::complex::Complex32;

use super::{Mode, State};

type MonteCarloF32 = dyn MonteCarlo<Output = f32> + Send + Sync;
type MonteCarloComplex32 = dyn MonteCarlo<Output = Complex32> + Send + Sync;

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

#[derive(Default)]
pub struct MonteCarloCache {
    cache_real: HashMap<CacheKey, Box<MonteCarloF32>>,
    cache_complex: HashMap<CacheKey, Box<MonteCarloComplex32>>,
}

impl MonteCarloCache {
    pub fn get_or_create_f32(&mut self, state: &State) -> Option<&mut MonteCarloF32> {
        if state.mode().is_complex() {
            return None;
        }
        Some(
            self.cache_real
                .entry(CacheKey::from(state))
                .or_insert_with(|| match state.mode() {
                    Mode::RealSimple | Mode::RealFull => {
                        Box::new(AcceptReject::new(orbital::Real::new(*state.qn())))
                    }
                    Mode::Hybrid => Box::new(AcceptReject::new(orbital::hybrid::Hybrid::new(
                        state.hybrid_kind().archetype().clone(),
                    ))),
                    Mode::Complex => unreachable!(),
                })
                .as_mut(),
        )
    }

    pub fn get_or_create_complex32(&mut self, state: &State) -> Option<&mut MonteCarloComplex32> {
        if !state.mode().is_complex() {
            return None;
        }
        Some(
            self.cache_complex
                .entry(CacheKey::from(state))
                .or_insert_with(|| match state.mode() {
                    Mode::Complex => {
                        Box::new(AcceptReject::new(orbital::Complex::new(*state.qn())))
                    }
                    _ => unreachable!(),
                })
                .as_mut(),
        )
    }
}

pub static MONTE_CARLO_CACHE: LazyLock<Mutex<MonteCarloCache>> =
    LazyLock::new(|| Mutex::new(Default::default()));
