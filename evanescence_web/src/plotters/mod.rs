pub(crate) mod pointillist;
pub(crate) mod supplemental;

use evanescence_core::orbital::{self, Qn};

/// Yet another heuristic for scaling the cutoff value appropriately. As the number of lobes
/// increases, they attain increasingly small values, which require a lower cutoff to achieve
/// an adequate appearance (i.e., not showing too small of a portion).
pub(crate) fn isosurface_cutoff_heuristic(qn: &Qn) -> f32 {
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
