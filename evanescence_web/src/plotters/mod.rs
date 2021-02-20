pub(crate) mod pointillist;
pub(crate) mod supplemental;

use evanescence_core::orbital::{self, Qn};

fn isosurface_cutoff_heuristic(qn: &Qn) -> f32 {
    // Yet another heuristic for scaling the cutoff value appropriately. As the number of lobes
    // increases, they attain increasingly small values, which require a lower cutoff to achieve
    // an adequate appearance (i.e., not showing too small of a portion).
    let num_radial_nodes = orbital::Real::num_radial_nodes(qn);
    let num_angular_nodes = orbital::Real::num_angular_nodes(qn);
    let num_lobes = (num_radial_nodes + 1) * (num_angular_nodes + 1);
    let damping_factor = if num_radial_nodes == 0 && num_angular_nodes > 2 {
        0.06
    } else {
        0.012
    };
    0.003 / ((num_lobes as f32 - 1.0).powf(2.5) * damping_factor + 1.0)
}
