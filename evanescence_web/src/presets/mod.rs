mod hybrid_preset;
mod qn_preset;

use std::marker::PhantomData;

use derivative::Derivative;
use evanescence_core::orbital::hybrid::Kind;
use evanescence_core::orbital::Qn;
use serde::{Deserialize, Serialize};

#[derive(Derivative, Serialize, Deserialize)]
// `#[derive()]` gets confused by `PhantomData` and incorrectly enforces bounds on `T` itself.
// See https://github.com/rust-lang/rust/issues/26925.
#[derivative(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct Preset<T> {
    idx: usize,
    phantom: PhantomData<T>,
}

/// Helper trait for linking a given [`Preset<T>`] to its list of presets.
pub trait PresetLibrary {
    type Item;
    fn library() -> &'static [Self::Item];
}

impl<T> Preset<T>
where
    Self: PresetLibrary<Item = T>,
    T: 'static + PartialEq,
{
    fn new(idx: usize) -> Self {
        Self {
            idx,
            phantom: PhantomData,
        }
    }

    pub fn presets() -> Vec<Self> {
        (0..Self::library().len()).map(Self::new).collect()
    }

    pub fn item(&self) -> &'static T {
        Self::library()
            .get(self.idx)
            .unwrap_or_else(|| panic!("preset with index {} does not exist", self.idx))
    }

    pub fn try_find(value: &T) -> Option<Self> {
        Self::library()
            .iter()
            .position(|item| item == value)
            .map(Self::new)
    }
}

pub type QnPreset = Preset<Qn>;
pub type HybridPreset = Preset<Kind>;
