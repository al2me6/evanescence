pub mod pointillist;
pub mod supplemental;

use evanescence_core::numerics::{Evaluate, EvaluateBounded};
use evanescence_core::orbital::hybrid::{Hybrid, Kind};
use evanescence_core::orbital::{Qn, Real};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

use crate::plotly::color::color_scales;
use crate::plotly::isosurface::{self, Isosurface};

/// Yet another heuristic for scaling the cutoff value appropriately. As the number of lobes
/// increases, they attain increasingly small values, which require a lower cutoff to achieve
/// an adequate appearance (i.e., not showing too small of a portion).
///
/// Note that this cutoff value is for the wavefunction, not the probability density.
pub fn isosurface_cutoff_heuristic_real(qn: &Qn) -> f32 {
    let num_radial_nodes = Real::num_radial_nodes(*qn);
    let num_angular_nodes = Real::num_angular_nodes(*qn);
    let num_lobes = (num_radial_nodes + 1) * (num_angular_nodes + 1);
    let damping_factor = if num_radial_nodes == 0 {
        0.3 + 0.02 * num_angular_nodes as f32
    } else if num_angular_nodes == 0 {
        0.4 + 0.08 * num_radial_nodes as f32
    } else {
        0.085
    };
    0.006 / ((num_lobes as f32 - 1.0).powi(2) * damping_factor + 1.0)
}

pub fn isosurface_cutoff_heuristic_hybrid(kind: &Kind) -> f32 {
    let mixture = kind.mixture();
    mixture
        .iter()
        .flat_map(|(&l, &count)| itertools::repeat_n(l, count as usize))
        .map(|l| Qn::new(kind.n(), l, 0).unwrap())
        .map(|qn| isosurface_cutoff_heuristic_real(&qn))
        .sum::<f32>()
        / mixture.values().sum::<u32>() as f32
        * 1.8
}

fn compute_isosurface_hybrid(kind: &Kind, idx: usize, quality: Quality) -> Isosurface<'static> {
    let lc = &kind.combinations()[idx];
    let hybrid = Hybrid::new(lc.clone());
    let (x, y, z, value) = hybrid
        .evaluate_in_region(
            // Manually shrink the extent sampled for higher quality.
            hybrid.bound() * 0.82,
            quality.grid_3d(),
        )
        .into_components();
    let cutoff = isosurface_cutoff_heuristic_hybrid(kind);

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

    pub fn point_cloud(self) -> usize {
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
