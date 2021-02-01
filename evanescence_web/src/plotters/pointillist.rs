use std::convert::TryInto;

use evanescence_core::{
    geometry::{ComponentForm, Plane},
    monte_carlo::MonteCarlo,
    numerics::normalize,
    orbital::{self, wavefunctions, Orbital},
};
use wasm_bindgen::JsValue;

use crate::plotly::{
    color::{color_scales, ColorBar, ColorScale},
    layout::Anchor,
    scatter_3d::Marker,
    surface::Contours,
};
use crate::plotly::{Isosurface, Scatter3D, Surface};
use crate::state::State;
use crate::utils::min_max;

fn isosurface(
    simulation: ComponentForm<f32>,
    correct_instability: bool,
    color_scale: ColorScale,
) -> JsValue {
    let (x, y, z, mut value) = simulation.into_components();
    if correct_instability {
        // HACK: We take the "signed square root", i.e. `sgn(x) * sqrt(|x|)` here to alleviate
        // numerical instability/artifacting by amplifying any deviations from zero. However,
        // this also results in crinkly-looking surfaces.
        value = value
            .into_iter()
            .map(|v| v.signum() * v.abs().sqrt())
            .collect();
    }
    Isosurface {
        x,
        y,
        z,
        value,
        color_scale,
        opacity: 0.075,
        ..Default::default()
    }
    .into()
}

pub(crate) fn real(state: &State) -> JsValue {
    let simulation = orbital::Real::monte_carlo_simulate(state.qn, state.quality);
    let (x, y, z, values) = simulation.into_components();

    let values_abs: Vec<_> = values.iter().map(|&v| v.abs()).collect();
    let (min_abs, max_abs) = min_max(values_abs.iter());

    Scatter3D {
        x,
        y,
        z,
        marker: Marker {
            size: values_abs
                .into_iter()
                .map(|v| normalize(min_abs..=max_abs, 0.2..=5.0, v))
                .collect(),
            color: values,
            show_scale: true,
            color_bar: ColorBar {
                x: 0.0,
                x_anchor: Anchor::Right,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
    .into()
}

pub(crate) fn radial_nodes(state: &State) -> JsValue {
    isosurface(
        orbital::sample_region_for::<wavefunctions::Radial>(
            state.qn,
            state.quality.for_isosurface(),
            // Shrink the extent plotted since radial nodes are found in the central
            // part of the full extent only. This is a heuristic that has been verified
            // to cover all radial nodes from `n` = 2 through 8.
            Some(state.qn.n() as f32 * 0.05 + 0.125),
        ),
        false,
        color_scales::GREENS,
    )
}

pub(crate) fn angular_nodes(state: &State) -> JsValue {
    let qn = state.qn;
    isosurface(
        orbital::sample_region_for::<wavefunctions::RealSphericalHarmonic>(
            qn,
            state.quality.for_isosurface(),
            None,
        ),
        qn.l() >= 4 && qn.m().abs() >= 4,
        color_scales::PURP,
    )
}

pub(crate) fn cross_section_indicator(state: &State) -> JsValue {
    let plane: Plane = state.extra_visualization.try_into().unwrap();
    let (x, y, z) = plane
        .four_points_as_xy_value(orbital::Real::estimate_radius(state.qn))
        .into_components();
    Surface {
        x,
        y,
        z,
        opacity: 0.2,
        show_scale: false,
        color_scale: color_scales::ORANGES,
        surface_color: Some(vec![vec![0.0; 2]; 2]),
        contours: Some(Contours::default()),
        ..Default::default()
    }
    .into()
}
