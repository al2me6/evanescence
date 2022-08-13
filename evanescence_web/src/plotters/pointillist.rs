use std::default::default;
use std::f32::consts::{PI, TAU};

use evanescence_core::geometry::storage::grid_values_3::CoordinatePlane3;
use evanescence_core::geometry::storage::ComponentForm3;
use evanescence_core::numerics::function::{Function3Ext, Function3InOriginCenteredRegionExt};
use evanescence_core::numerics::{self, Function};
use evanescence_core::orbital::hybrid::Hybrid;
use evanescence_core::orbital::Real;
use na::Point3;
use wasm_bindgen::JsValue;

use crate::plotly::color::{self, color_scales, ColorBar};
use crate::plotly::layout::{Anchor, Title};
use crate::plotly::scatter_3d::Marker;
use crate::plotly::{Isosurface, Scatter3D, Surface};
use crate::state::cache::MONTE_CARLO_CACHE;
use crate::state::{Mode, State};
use crate::utils;

pub fn real(state: &State) -> JsValue {
    assert!([Mode::RealSimple, Mode::RealFull, Mode::Hybrid].contains(&state.mode()));

    let (x, y, z, values) = state.monte_carlo_simulate_real().into_components();

    // Special handling for s orbitals.
    let min_point_size = if state.mode().is_real_or_simple() && state.qn().l() == 0 {
        0.4
    } else {
        0.2
    };

    let mut values_abs: Vec<_> = values.iter().map(|&v| v.abs()).collect();
    let max_abs = *utils::partial_max(&values_abs).unwrap();
    numerics::normalize_collection(0.0..=max_abs, min_point_size..=4.0, &mut values_abs);

    Scatter3D {
        x,
        y,
        z,
        marker: Marker {
            size: values_abs,
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

    let (x, y, z, values) = MONTE_CARLO_CACHE
        .lock()
        .unwrap()
        .request_complex32(state.into(), state.quality().point_cloud())
        .unwrap()
        .collect::<ComponentForm3<_>>()
        .into_components();

    let (mut moduli, arguments) = utils::split_moduli_arguments(&values);

    // Special handling for s orbitals.
    let min_point_size = if state.qn().l() == 0 { 0.8 } else { 0.4 };
    let max_modulus = *utils::partial_max(&moduli).unwrap();
    numerics::normalize_collection(0.0..=max_modulus, min_point_size..=4.0, &mut moduli);

    Scatter3D {
        x,
        y,
        z,
        marker: Marker {
            size: moduli,
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
    let theta_samples = numerics::linspace(0.0..=PI, samples);
    let phi_samples = numerics::linspace(0.0..=TAU, samples);
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

    Real::radial_node_positions(*state.qn())
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

/// A square of side length 2 centered at the origin.
fn polar_square(theta: f32) -> f32 {
    f32::min(theta.cos().recip().abs(), theta.sin().recip().abs())
}

struct VerticalCone {
    theta: f32,
}

impl VerticalCone {
    fn new(theta: f32) -> Self {
        Self { theta }
    }
}

impl Function<3, Point3<f32>> for VerticalCone {
    type Output = f32;

    fn evaluate(&self, point: &Point3<f32>) -> Self::Output {
        // Note that the z values of passed points are ignored!
        (point.coords.x * point.coords.x + point.coords.y * point.coords.y).sqrt()
            / self.theta.tan()
    }
}

pub fn nodes_angular(state: &State) -> Vec<JsValue> {
    const NUM_POINTS: usize = 75;

    assert!(state.mode().is_real_or_simple());

    let qn = state.qn();
    let bound = state.bound();
    Iterator::chain(
        Real::conical_node_angles(qn.into())
            .into_iter()
            .map(|theta| {
                VerticalCone::new(theta)
                    .evaluate_on_plane(CoordinatePlane3::XY, bound, NUM_POINTS)
                    .into_components()
            })
            .map(|(x, y, z)| Surface {
                x: Some(x),
                y: Some(y),
                z,
                surface_color: Some(vec![vec![0.0_f32; NUM_POINTS]; NUM_POINTS]),
                ..default()
            }),
        Real::planar_node_angles(qn.into()).into_iter().map(|phi| {
            let r = bound;
            let r_square = polar_square(phi);
            let (x1, y1) = (r * phi.cos() * r_square, r * phi.sin() * r_square);
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
    let plane: CoordinatePlane3 = state.supplement().try_into().unwrap();
    let (x, y, z) = plane.square_wrt_xy_plane(state.bound()).into_components();
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
    assert!(state.mode().is_hybrid());

    let (x, y, z, value) = Hybrid::new(state.hybrid_kind().archetype().clone())
        .sample_region(state.quality().grid_3d())
        .into_components();

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
