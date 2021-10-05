use std::default::default;
use std::f32::consts::PI;

use evanescence_core::geometry::{ComponentForm, Plane, Point, PointValue};
use evanescence_core::monte_carlo::MonteCarlo;
use evanescence_core::numerics::{self, Evaluate, EvaluateBounded};
use evanescence_core::orbital::hybrid::Hybrid;
use evanescence_core::orbital::molecular::{Molecular, OffsetQnWeight};
use evanescence_core::orbital::{atomic, Complex};
use wasm_bindgen::JsValue;

use crate::plotly::color::{self, color_scales, ColorBar};
use crate::plotly::layout::{Anchor, Title};
use crate::plotly::scatter_3d::Marker;
use crate::plotly::surface::Contours;
use crate::plotly::{Isosurface, Scatter3D, Surface};
use crate::state::{Mode, State};
use crate::utils;

pub(crate) fn real(state: &State) -> JsValue {
    assert!([Mode::RealSimple, Mode::Real, Mode::Hybrid, Mode::Mo].contains(&state.mode()));

    let (x, y, z, values) = state.monte_carlo_simulate_real().into_components();
    let values_abs: Vec<_> = values.iter().map(|&v| v.abs()).collect();
    let max_abs = *utils::partial_max(&values_abs).unwrap();

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
                .map(|v| numerics::normalize(0.0..=max_abs, min_point_size..=4.0, v))
                .collect(),
            color: values,
            show_scale: true,
            color_bar: ColorBar {
                x: -0.02,
                x_anchor: Anchor::Right,
                ..default()
            },
            ..default()
        },
        show_legend: false,
        ..default()
    }
    .into()
}

pub(crate) fn complex(state: &State) -> JsValue {
    assert!(state.mode().is_complex());

    let simulation = Complex::monte_carlo_simulate(state.qn(), state.quality(), true);
    let (x, y, z, values) = simulation.into_components();

    let moduli: Vec<_> = values.iter().map(|v| v.norm()).collect();
    let arguments: Vec<_> = values.iter().map(|v| v.arg()).collect();
    let max_modulus = *utils::partial_max(&moduli).unwrap();

    // Special handling for s orbitals.
    let min_point_size = if state.qn().l() == 0 { 0.8 } else { 0.4 };

    Scatter3D {
        x,
        y,
        z,
        marker: Marker {
            size: moduli
                .into_iter()
                .map(|m| numerics::normalize(0.0..=max_modulus, min_point_size..=3.0, m))
                .collect(),
            color: arguments,
            color_scale: color_scales::PHASE,
            show_scale: true,
            c_min: Some(-PI),
            c_mid: 0.0,
            c_max: Some(PI),
            color_bar: ColorBar {
                x: -0.02,
                x_anchor: Anchor::Right,
                tick_vals: Some(color::PHASE_BAR_TICKS),
                tick_text: Some(color::PHASE_BAR_LABELS),
                title: Some(Title {
                    text: "Phase",
                    ..default()
                }),
                ..default()
            },
            ..default()
        },
        show_legend: false,
        ..default()
    }
    .into()
}

pub(crate) fn nodes_radial(state: &State) -> JsValue {
    assert!(state.mode().is_real_or_simple());

    let (x, y, z, value) = atomic::Radial::evaluate_in_region(
        &state.qn().into(),
        // Shrink the extent plotted since radial nodes are found in the central part of the
        // full extent only. This is a heuristic that has been verified to cover all radial
        // nodes from `n` = 2 through 8.
        state.bound() * (state.qn().n() as f32 * 0.06 + 0.125),
        state.quality().for_isosurface(),
    )
    .into_components();

    Isosurface {
        x,
        y,
        z,
        value,
        color_scale: color_scales::GREENS,
        opacity: 0.125,
        ..default()
    }
    .into()
}

pub(crate) fn nodes_angular(state: &State) -> JsValue {
    assert!(state.mode().is_real_or_simple());

    let qn = state.qn();
    let (x, y, z, mut value) = atomic::RealSphericalHarmonic::evaluate_in_region(
        &qn.into(),
        state.bound(),
        state.quality().for_isosurface(),
    )
    .into_components();
    if qn.l() >= 4 && qn.m().abs() >= 4 {
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
        color_scale: color_scales::PURP,
        opacity: 0.15,
        ..default()
    }
    .into()
}

pub(crate) fn cross_section_indicator(state: &State) -> JsValue {
    let plane: Plane = state.supplement().try_into().unwrap();
    let (x, y, z) = plane
        .four_points_as_xy_value(state.bound())
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
        ..default()
    }
    .into()
}

pub(crate) fn silhouettes(state: &State) -> Vec<JsValue> {
    assert!(state.mode().is_hybrid());
    let kind = state.hybrid_kind();

    let rescale_factor = *kind.mixture().keys().max().expect("kind is not empty") as f32;
    let rescale_factor = 5.0 / (rescale_factor.powi(2) * 0.3 + 1.0);

    kind.iter()
        .enumerate()
        .map(|(idx, lc)| {
            let surface = super::compute_isosurface_hybrid(kind, idx, state.quality());
            // Shrink the surface for silhouettes so they overlap less.
            let cutoff = surface.iso_max * rescale_factor;
            Isosurface {
                iso_min: -cutoff,
                iso_max: cutoff,
                c_min: Some(-cutoff * 1.4),
                c_max: Some(cutoff * 1.4),
                opacity: if lc == kind.archetype() { 0.35 } else { 0.15 },
                ..surface
            }
            .into()
        })
        .collect()
}

pub(crate) fn nodes_combined(state: &State) -> JsValue {
    assert!(state.mode().is_hybrid() || state.mode().is_mo());

    let (x, y, z, value) = if state.mode().is_hybrid() {
        Hybrid::sample_region(
            state.hybrid_kind().archetype(),
            state.quality().for_isosurface(),
        )
        .into_components()
    } else {
        Molecular::sample_region(&state.lcao(), state.quality().for_isosurface()).into_components()
    };

    Isosurface {
        x,
        y,
        z,
        value,
        color_scale: color_scales::PURP,
        opacity: 0.125,
        ..default()
    }
    .into()
}

pub(crate) fn nucleus_markers(state: &State) -> JsValue {
    const MARKER_SIZE: f32 = 15.0;

    assert!(state.mode().is_mo());

    let offsets = state
        .lcao()
        .weights
        .iter()
        .map(|OffsetQnWeight { offset, .. }| offset)
        .map(Point::from)
        .map(|pt| PointValue(pt, MARKER_SIZE))
        .collect::<Vec<_>>();
    let (x, y, z, v) = ComponentForm::from(offsets).into_components();

    Scatter3D {
        x,
        y,
        z,
        marker: Marker {
            size: v.clone(),
            color: v,
            color_scale: color_scales::GREENS,
            c_min: Some(0.0),
            c_max: Some(MARKER_SIZE * 1.25),
            ..default()
        },
        show_legend: false,
        ..default()
    }
    .into()
}
