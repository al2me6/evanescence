pub(crate) mod pointillist;
pub(crate) mod supplemental;

use evanescence_core::geometry::ComponentForm;
use evanescence_core::monte_carlo::Quality;
use evanescence_core::numerics::Evaluate;
use evanescence_core::orbital::{self, LinearCombination, Orbital, Qn};

use crate::plotly::color::color_scales;
use crate::plotly::isosurface::{self, Isosurface};

/// Yet another heuristic for scaling the cutoff value appropriately. As the number of lobes
/// increases, they attain increasingly small values, which require a lower cutoff to achieve
/// an adequate appearance (i.e., not showing too small of a portion).
///
/// Note that this cutoff value is for the wavefunction, not the probability density.
pub(crate) fn isosurface_cutoff_heuristic_real(qn: &Qn) -> f32 {
    let num_radial_nodes = orbital::Real::num_radial_nodes(qn);
    let num_angular_nodes = orbital::Real::num_angular_nodes(qn);
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

pub(crate) fn isosurface_cutoff_heuristic_hybrid(lc: &LinearCombination) -> f32 {
    lc.iter()
        .map(|(qn, _)| qn)
        .map(isosurface_cutoff_heuristic_real)
        .sum::<f32>()
        / lc.count() as f32
        * 4.0
}

fn compute_isosurface_hybrid(lc: &LinearCombination, quality: Quality) -> Isosurface {
    let (x, y, z, value) = ComponentForm::from(orbital::Hybrid::evaluate_in_region(
        lc,
        // Manually shrink the extent sampled for higher quality.
        orbital::Hybrid::estimate_radius(lc) * 0.75,
        quality.for_isosurface(),
    ))
    .into_components();
    let cutoff = isosurface_cutoff_heuristic_hybrid(lc);

    Isosurface {
        x,
        y,
        z,
        value,
        iso_min: -cutoff,
        iso_max: cutoff,
        surface: isosurface::Surface { count: 2 },
        color_scale: color_scales::RED_BLUE_REVERSED,
        ..Default::default()
    }
}
