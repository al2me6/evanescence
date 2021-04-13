use std::convert::TryInto;
use std::default::default;
use std::f32::consts::PI;

use evanescence_core::geometry::Plane;
use evanescence_core::orbital::{self, Orbital, RadialPlot};
use wasm_bindgen::JsValue;

use super::isosurface_cutoff_heuristic;
use crate::plotly::color::{self, color_scales, ColorBar};
use crate::plotly::layout::{Axis, Scene, Title};
use crate::plotly::scatter::Line;
use crate::plotly::surface::{Contour, Contours, Project};
use crate::plotly::{isosurface, Isosurface, Layout, Scatter, Surface};
use crate::state::State;
use crate::utils::{self, b16_colors};

pub(crate) fn radial(state: &State) -> (JsValue, JsValue) {
    const NUM_POINTS: usize = 600;

    assert!(state.mode().is_real_or_simple() || state.mode().is_complex());

    let variant: RadialPlot = state.supplement().try_into().unwrap();
    let function_expr = match variant {
        RadialPlot::Wavefunction => "R(r)",
        RadialPlot::ProbabilityDistribution => "r²R(r)²",
    };
    let variant_label = format!(
        "{} [ {} ]",
        utils::capitalize_words(state.supplement().to_string()),
        function_expr
    );

    let (x, y) = orbital::sample_radial(state.qn(), variant, NUM_POINTS);

    let trace = Scatter {
        x,
        y,
        line: Line {
            color: Some(
                b16_colors::BASE[match variant {
                    RadialPlot::Wavefunction => 0x0c,
                    RadialPlot::ProbabilityDistribution => 0x0a,
                }],
            ),
            ..default()
        },
        fill: matches!(variant, RadialPlot::ProbabilityDistribution).then(|| "tozeroy"),
        ..default()
    };

    let layout = Layout {
        ui_revision: Some(&variant_label),
        drag_mode_bool: Some(false),
        x_axis: Some(Axis {
            title: Some(Title {
                text: "r (Bohr radii)",
                standoff: Some(20),
            }),
            ticks: "outside",
            ..default()
        }),
        y_axis: Some(Axis {
            title: Some(Title {
                text: &variant_label,
                standoff: Some(20),
            }),
            range_mode: Some("tozero"),
            ticks: "outside",
            ..default()
        }),
        ..default()
    };

    (trace.into(), layout.into())
}

#[allow(clippy::too_many_lines)] // Plotly configuration is verbose.
pub(crate) fn cross_section(state: &State) -> (JsValue, JsValue) {
    const ZERO_THRESHOLD: f32 = 1E-7;

    let is_complex = state.mode().is_complex();
    let plane: Plane = state.supplement().try_into().unwrap();

    let (x, y, mut z, custom_color) = if is_complex {
        let (x, y, values) =
            orbital::Complex::sample_cross_section(state.qn(), plane, state.quality().for_grid())
                .into_components();
        let moduli = values
            .iter()
            .map(|row| row.iter().map(|v| v.norm()).collect())
            .collect();
        let arguments: Vec<Vec<_>> = values
            .iter()
            .map(|row| row.iter().map(|v| v.arg()).collect())
            .collect();
        (x, y, moduli, Some(arguments))
    } else {
        let (x, y, z) = state.sample_cross_section_real(plane).into_components();
        (x, y, z, None)
    };

    let max_abs = utils::partial_max(z.iter().flat_map(|row| row.iter()).map(|v| v.abs())).unwrap();
    if max_abs < ZERO_THRESHOLD {
        // Zero all values.
        z.iter_mut()
            .flat_map(|row| row.iter_mut())
            .for_each(|v| *v = 0.0);
    }

    let ui_revision = state.supplement().to_string();
    let (x_label, y_label) = plane.axes_names();
    let xyz_arguments = match plane {
        Plane::XY => "(x, y, 0)",
        Plane::YZ => "(0, y, z)",
        Plane::ZX => "(x, 0, z)",
    };
    let z_axis_title = if is_complex {
        format!("Wavefunction Modulus [ |ψ{}| ]", xyz_arguments)
    } else {
        format!("Wavefunction [ ψ{} ]", xyz_arguments)
    };
    let contour_max_abs = max_abs * 1.05;

    let trace = Surface {
        x,
        y,
        z,
        surface_color: custom_color,
        color_bar: if is_complex {
            ColorBar {
                tick_vals: Some(color::PHASE_BAR_TICKS),
                tick_text: Some(color::PHASE_BAR_LABELS),
                title: Some(Title {
                    text: "Phase",
                    ..default()
                }),
                ..default()
            }
        } else {
            ColorBar::default()
        },
        color_scale: if is_complex {
            color_scales::PHASE
        } else {
            color_scales::RED_BLUE_REVERSED
        },
        c_min: is_complex.then(|| -PI),
        c_mid: 0.0,
        c_max: is_complex.then(|| PI),
        contours: Some(Contours {
            z: Contour {
                show: Some(true),
                start: Some(-contour_max_abs),
                end: Some(contour_max_abs),
                // For up to 10 contour lines for each sign, plus one at zero.
                size: Some(contour_max_abs / 11.0),
                color: is_complex.then(|| b16_colors::BASE[0x03]), // Use default otherwise.
                use_color_map: Some(!is_complex),
                highlight: true,
                project: Some(Project {
                    z: true,
                    ..default()
                }),
                ..Default::default()
            },
            ..default()
        }),
        ..default()
    };

    let layout = Layout {
        ui_revision: Some(&ui_revision),
        scene: Some(Scene {
            x_axis: Axis {
                title: Some(Title {
                    text: x_label,
                    ..default()
                }),
                ..default()
            },
            y_axis: Axis {
                title: Some(Title {
                    text: y_label,
                    ..default()
                }),
                ..default()
            },
            z_axis: Axis {
                title: Some(Title {
                    text: &z_axis_title,
                    ..default()
                }),
                n_ticks: Some(6),
                ..default()
            },
            ..default()
        }),
        ..default()
    };

    (trace.into(), layout.into())
}

pub(crate) fn isosurface_3d(state: &State) -> (JsValue, JsValue) {
    assert!(state.mode().is_real_or_simple());

    let (x, y, z, value) =
        orbital::Real::sample_region(state.qn(), state.quality().for_isosurface() * 3 / 2)
            .into_components();

    let cutoff = isosurface_cutoff_heuristic(state.qn());

    let axis = Axis::with_extent(state.estimate_radius());
    let trace = Isosurface {
        x,
        y,
        z,
        value,
        iso_min: -cutoff,
        iso_max: cutoff,
        surface: isosurface::Surface { count: 2 },
        color_scale: color_scales::RED_BLUE_REVERSED,
        c_min: Some(-cutoff * 1.2),
        c_max: Some(cutoff * 1.2),
        opacity: if state.qn().l() == 0 { 0.5 } else { 1.0 },
        ..default()
    };

    let layout = Layout {
        ui_revision: Some("isosurface"),
        drag_mode_str: Some("orbit"),
        scene: Some(Scene {
            x_axis: axis,
            y_axis: axis,
            z_axis: axis,
            ..default()
        }),
        ..default()
    };

    (trace.into(), layout.into())
}
