pub mod pointillist;
pub mod supplemental;

use evanescence_core::geometry::region::BoundingRegion;
use evanescence_core::numerics::Function;
use evanescence_core::orbital::hybrid::{Hybrid, Kind};
use evanescence_core::orbital::Qn;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

use crate::plotly::color::color_scales;
use crate::plotly::isosurface::{self, Isosurface};
use crate::state::cache::MONTE_CARLO_CACHE;
use crate::state::MonteCarloParameters;

pub const ISOSURFACE_CUTOFF: f32 = 0.85;
const ISOSURFACE_SAMPLES: usize = Quality::High.point_cloud();

/// Note that this gives the |psi| cutoff, not psi squared.
fn compute_isosurface_cutoff_real(params: MonteCarloParameters) -> f32 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    const CUTOFF_IDX: usize = ((1. - ISOSURFACE_CUTOFF) * ISOSURFACE_SAMPLES as f32) as usize;

    let mut cache = MONTE_CARLO_CACHE.lock().unwrap();
    let monte_carlo_samples = cache.request_f32(params, ISOSURFACE_SAMPLES).unwrap();
    let mut psi_abs = monte_carlo_samples
        .values()
        .iter()
        .map(|psi| psi.abs())
        .collect_vec();
    psi_abs.sort_by(f32::total_cmp);
    psi_abs[CUTOFF_IDX]
}

/// Note that this cutoff value is for the wavefunction, not the probability density.
pub fn isosurface_cutoff_atomic_real(qn: &Qn) -> f32 {
    compute_isosurface_cutoff_real(MonteCarloParameters::AtomicReal(*qn))
}

pub fn isosurface_cutoff_hybrid(kind: &'static Kind) -> f32 {
    compute_isosurface_cutoff_real(MonteCarloParameters::Hybrid(kind))
}

fn compute_isosurface_hybrid(
    kind: &'static Kind,
    idx: usize,
    quality: Quality,
) -> Isosurface<'static> {
    let lc = &kind.combinations()[idx];
    let hybrid = Hybrid::new(lc.clone());
    let ([x, y, z], value) = hybrid
        .sample_from_origin_centered_hypercube(
            hybrid.bounding_region().radius * 0.85 * 2.,
            quality.grid_3d(),
        )
        .into_components();
    let cutoff = isosurface_cutoff_hybrid(kind);

    Isosurface {
        x,
        y,
        z,
        value,
        iso_min: -cutoff,
        iso_max: cutoff,
        surface: isosurface::Surface { count: 2 },
        color_scale: color_scales::RD_BU_R,
        ..Default::default()
    }
}

#[derive(
    Clone, Copy, PartialEq, Eq, Debug, Display, EnumString, EnumIter, Serialize, Deserialize,
)]
pub enum Quality {
    Minimum,
    Low,
    Medium,
    High,
    VeryHigh,
    Extreme,
}

impl Default for Quality {
    fn default() -> Self {
        Self::Low
    }
}

impl Quality {
    pub fn to_text(self) -> String {
        match self {
            Self::VeryHigh => "Very high".into(),
            quality => quality.to_string(),
        }
    }
}

#[allow(
    clippy::cast_possible_truncation, // Discriminants are small enough.
    clippy::cast_sign_loss, // Roots of positive numbers are positive.
    clippy::integer_division, // Intentional.
)]
impl Quality {
    pub fn grid_2d(self) -> usize {
        ((self.point_cloud() as f32).sqrt() * 0.75) as usize | 0b1 // Force the number to be odd.
    }

    pub fn grid_3d(self) -> usize {
        (self.point_cloud() as f32 * 4.0).cbrt() as usize | 0b1 // Force the number to be odd.
    }

    pub const fn point_cloud(self) -> usize {
        // These values have been empirically observed to produce reasonable results.
        match self {
            Self::Minimum => 1 << 12,
            Self::Low => 1 << 13,
            Self::Medium => 1 << 14,
            Self::High => 1 << 15,
            Self::VeryHigh => 1 << 16,
            Self::Extreme => 1 << 17,
        }
    }
}
