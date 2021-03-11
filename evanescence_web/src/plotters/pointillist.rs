use std::convert::TryInto;
use std::f32::consts::{FRAC_PI_2, PI};
use std::iter;

use evanescence_core::geometry::{ComponentForm, Plane};
use evanescence_core::monte_carlo::MonteCarlo;
use evanescence_core::numerics::{normalize, Evaluate};
use evanescence_core::orbital::{self, wavefunctions};
use orbital::Orbital;
use wasm_bindgen::JsValue;

use super::isosurface_cutoff_heuristic;
use crate::plotly::color::{color_scales, ColorBar, ColorScale};
use crate::plotly::layout::{Anchor, Title};
use crate::plotly::scatter_3d::Marker;
use crate::plotly::surface::Contours;
use crate::plotly::{isosurface, Isosurface, Scatter3D, Surface};
use crate::state::State;
use crate::utils::partial_max;

fn isosurface(
    simulation: ComponentForm<f32>,
    color_scale: ColorScale,
    correct_instability: bool,
) -> JsValue {
    let (x, y, z, mut value) = simulation.into_components();
    if correct_instability {
        // HACK: We take the "signed square root", i.e. `sgn(x) * sqrt(|x|)` here to alleviate
        // numerical instability/artifacting by amplifying any deviations from zero. However,
        // this also results in crinkly-looking surfaces.
        value
            .iter_mut()
            .for_each(|v| *v = v.signum() * v.abs().sqrt());
    }
    Isosurface {
        x,
        y,
        z,
        value,
        color_scale,
        opacity: 0.125,
        ..Default::default()
    }
    .into()
}

pub(crate) fn real(state: &State) -> JsValue {
    assert!(state.mode().is_real_or_simple() || state.mode().is_hybrid());

    let simulation = state.monte_carlo_simulate_real();
    let (x, y, z, values) = simulation.into_components();

    let values_abs: Vec<_> = values.iter().map(|&v| v.abs()).collect();
    let max_abs = *partial_max(&values_abs).unwrap();

    // Special handling for s orbitals.
    let min_point_size = if state.mode().is_real_or_simple() && state.qn().l() == 0 {
        0.6
    } else {
        0.3
    };

    Scatter3D {
        x,
        y,
        z,
        marker: Marker {
            size: values_abs
                .into_iter()
                .map(|v| normalize(0.0..=max_abs, min_point_size..=4.0, v))
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

pub(crate) fn complex(state: &State) -> JsValue {
    assert!(state.mode().is_complex());

    let simulation = orbital::Complex::monte_carlo_simulate(state.qn(), state.quality(), true);
    let (x, y, z, values) = simulation.into_components();

    let moduli: Vec<_> = values.iter().map(|v| v.norm()).collect();
    let arguments: Vec<_> = values.iter().map(|v| v.arg()).collect();
    let max_modulus = *partial_max(&moduli).unwrap();

    // Special handling for s orbitals.
    let min_point_size = if state.qn().l() == 0 { 0.8 } else { 0.4 };

    Scatter3D {
        x,
        y,
        z,
        marker: Marker {
            size: moduli
                .into_iter()
                .map(|m| normalize(0.0..=max_modulus, min_point_size..=3.0, m))
                .collect(),
            color: arguments,
            color_scale: color_scales::PHASE,
            show_scale: true,
            c_min: Some(-PI),
            c_mid: 0.0,
            c_max: Some(PI),
            color_bar: ColorBar {
                x: 0.0,
                x_anchor: Anchor::Right,
                tick_vals: Some(&[-PI, -FRAC_PI_2, 0.0, FRAC_PI_2, PI]),
                tick_text: Some(&["−π", "−π/2", "0", "π/2", "π"]),
                title: Some(Title {
                    text: "Phase",
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
    .into()
}

pub(crate) fn radial_nodes(state: &State) -> JsValue {
    assert!(state.mode().is_real_or_simple());

    isosurface(
        wavefunctions::Radial::evaluate_in_region(
            &state.qn().into(),
            // Shrink the extent plotted since radial nodes are found in the central part of the
            // full extent only. This is a heuristic that has been verified to cover all radial
            // nodes from `n` = 2 through 8.
            orbital::Real::estimate_radius(state.qn()) as f32
                * (state.qn().n() as f32 * 0.06 + 0.125),
            state.quality().for_isosurface(),
        )
        .into(),
        color_scales::GREENS,
        false,
    )
}

pub(crate) fn angular_nodes(state: &State) -> JsValue {
    assert!(state.mode().is_real_or_simple());

    let qn = state.qn();
    isosurface(
        wavefunctions::RealSphericalHarmonic::evaluate_in_region(
            &qn.into(),
            orbital::Real::estimate_radius(qn),
            state.quality().for_isosurface(),
        )
        .into(),
        color_scales::PURP,
        qn.l() >= 4 && qn.m().abs() >= 4,
    )
}

pub(crate) fn cross_section_indicator(state: &State) -> JsValue {
    assert!(state.mode().is_real_or_simple() || state.mode().is_hybrid());

    let plane: Plane = state.supplement().try_into().unwrap();
    let (x, y, z) = plane
        .four_points_as_xy_value(state.estimate_radius())
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

pub(crate) fn silhouettes(state: &State) -> Vec<JsValue> {
    assert!(state.mode().is_hybrid());

    state
        .hybrid_kind()
        .rotations()
        .iter()
        .chain(iter::once(state.hybrid_kind().principal()))
        .map(|orbital| {
            let (x, y, z, value) = ComponentForm::from(orbital::Hybrid::evaluate_in_region(
                orbital,
                // Manually shrink the extent sampled for higher quality.
                orbital::Hybrid::estimate_radius(orbital) * 0.6,
                state.quality().for_isosurface(),
            ))
            .into_components();

            let cutoff = orbital
                .iter()
                .map(|(qn, _)| qn)
                .map(isosurface_cutoff_heuristic)
                .sum::<f32>()
                / orbital.count() as f32
                * 6.0;

            Isosurface {
                x,
                y,
                z,
                value,
                iso_min: -cutoff,
                iso_max: cutoff,
                surface: isosurface::Surface { count: 2 },
                color_scale: color_scales::RED_BLUE_REVERSED,
                c_min: Some(-cutoff * 1.4),
                c_max: Some(cutoff * 1.4),
                opacity: if orbital == state.hybrid_kind().principal() {
                    0.3
                } else {
                    0.15
                },
                ..Default::default()
            }
            .into()
        })
        .collect()
}
