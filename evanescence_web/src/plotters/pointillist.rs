use std::default::default;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

use evanescence_core::geometry::{ComponentForm, Linspace, Plane, Point, PointValue};
use evanescence_core::monte_carlo::MonteCarlo;
use evanescence_core::numerics::{self, Evaluate, EvaluateBounded};
use evanescence_core::orbital::hybrid::Hybrid;
use evanescence_core::orbital::molecular::{LcaoAtom, Molecular};
use evanescence_core::orbital::{Complex, Real1};
use wasm_bindgen::JsValue;

use crate::plotly::color::{self, color_scales, ColorBar};
use crate::plotly::layout::{Anchor, Title};
use crate::plotly::scatter_3d::Marker;
use crate::plotly::{Isosurface, Scatter3D, Surface};
use crate::state::{Mode, State};
use crate::utils;

pub fn real(state: &State) -> JsValue {
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

pub fn complex(state: &State) -> JsValue {
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

fn parametric_sphere(r: f32, samples: usize) -> [Vec<Vec<f32>>; 3] {
    let (mut x, mut y, mut z) = (
        Vec::with_capacity(samples),
        Vec::with_capacity(samples),
        Vec::with_capacity(samples),
    );
    let theta_samples = (0.0..=PI).linspace(samples);
    let phi_samples = (0.0..=TAU).linspace(samples);
    for theta in theta_samples {
        let (mut x_row, mut y_row, mut z_row) = (
            Vec::with_capacity(samples),
            Vec::with_capacity(samples),
            Vec::with_capacity(samples),
        );
        for phi in phi_samples.clone() {
            x_row.push(r * theta.sin() * phi.cos());
            y_row.push(r * theta.sin() * phi.sin());
            z_row.push(r * theta.cos());
        }
        x.push(x_row);
        y.push(y_row);
        z.push(z_row);
    }
    [x, y, z]
}

pub fn nodes_radial(state: &State) -> Vec<JsValue> {
    const NUM_POINTS: usize = 40;

    assert!(state.mode().is_real_or_simple());

    Real1::radial_node_positions(*state.qn())
        .into_iter()
        .map(|r| parametric_sphere(r, NUM_POINTS))
        .map(|[x, y, z]| {
            Surface {
                x_parametric: Some(x),
                y_parametric: Some(y),
                z,
                surface_color: Some(vec![vec![0.0_f32; NUM_POINTS]; NUM_POINTS]),
                color_scale: color_scales::GREENS,
                show_scale: false,
                opacity: 0.125,
                contours: Some(default()),
                ..default()
            }
            .into()
        })
        .collect()
}

fn radius_to_square_multiplier(mut theta: f32) -> f32 {
    theta = theta.abs() % FRAC_PI_2;
    if theta > FRAC_PI_4 {
        theta = FRAC_PI_2 - theta;
    }
    1.0 / theta.cos()
}

struct VerticalCone;

impl Evaluate for VerticalCone {
    type Output = f32;
    type Parameters = f32;

    fn evaluate(params: &Self::Parameters, point: &Point) -> Self::Output {
        // Note that the z values of passed points are ignored!
        (point.x() * point.x() + point.y() * point.y()).sqrt() / params.tan()
    }
}

pub fn nodes_angular(state: &State) -> Vec<JsValue> {
    const NUM_POINTS: usize = 75;

    assert!(state.mode().is_real_or_simple());

    let qn = state.qn();
    let bound = state.bound();
    Iterator::chain(
        Real1::conical_node_angles(qn.into())
            .into_iter()
            .map(|theta| {
                VerticalCone::evaluate_on_plane(&theta, Plane::XY, bound, NUM_POINTS)
                    .into_components()
            })
            .map(|(x, y, z)| Surface {
                x: Some(x),
                y: Some(y),
                z,
                surface_color: Some(vec![vec![0.0_f32; NUM_POINTS]; NUM_POINTS]),
                ..default()
            }),
        Real1::planar_node_angles(qn.into()).into_iter().map(|phi| {
            let r = bound;
            let mult = radius_to_square_multiplier(phi);
            let (x1, y1) = (r * mult * phi.cos(), r * mult * phi.sin());
            let (x2, y2) = (-x1, -y1);
            Surface {
                x_parametric: Some(vec![vec![x1, x2]; 2]),
                y_parametric: Some(vec![vec![y1, y2]; 2]),
                z: vec![vec![-r, -r], vec![r, r]],
                surface_color: Some(vec![vec![0.0, 0.0]; 2]),
                ..default()
            }
        }),
    )
    .map(|srf| {
        Surface {
            color_scale: color_scales::PURP,
            show_scale: false,
            opacity: 0.15,
            contours: Some(default()),
            ..srf
        }
        .into()
    })
    .collect()
}

pub fn cross_section_indicator(state: &State) -> JsValue {
    let plane: Plane = state.supplement().try_into().unwrap();
    let (x, y, z) = plane
        .four_points_as_xy_value(state.bound())
        .into_components();
    Surface {
        x: Some(x),
        y: Some(y),
        z,
        opacity: 0.2,
        show_scale: false,
        color_scale: color_scales::ORANGES,
        surface_color: Some(vec![vec![0.0, 0.0]; 2]),
        contours: Some(default()),
        ..default()
    }
    .into()
}

pub fn silhouettes(state: &State) -> Vec<JsValue> {
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

pub fn nodes_combined(state: &State) -> JsValue {
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

pub fn nucleus_markers(state: &State) -> JsValue {
    const MARKER_SIZE: f32 = 15.0;

    assert!(state.mode().is_mo());

    let offsets = state
        .lcao()
        .combination
        .iter()
        .map(|LcaoAtom { offset, .. }| offset)
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
