pub(crate) mod pointillist;
pub(crate) mod supplemental;

use evanescence_core::monte_carlo::Quality;
use evanescence_core::numerics::{Evaluate, EvaluateBounded};
use evanescence_core::orbital::hybrid::Kind;
use evanescence_core::orbital::{Hybrid, Qn, Real};

use crate::plotly::color::color_scales;
use crate::plotly::isosurface::{self, Isosurface};

/// Yet another heuristic for scaling the cutoff value appropriately. As the number of lobes
/// increases, they attain increasingly small values, which require a lower cutoff to achieve
/// an adequate appearance (i.e., not showing too small of a portion).
///
/// Note that this cutoff value is for the wavefunction, not the probability density.
pub(crate) fn isosurface_cutoff_heuristic_real(qn: &Qn) -> f32 {
    let num_radial_nodes = Real::num_radial_nodes(qn);
    let num_angular_nodes = Real::num_angular_nodes(qn);
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

pub(crate) fn isosurface_cutoff_heuristic_hybrid(kind: &Kind) -> f32 {
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
    let (x, y, z, value) = Hybrid::evaluate_in_region(
        lc,
        // Manually shrink the extent sampled for higher quality.
        Hybrid::bound(lc) * 0.82,
        quality.for_isosurface(),
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
