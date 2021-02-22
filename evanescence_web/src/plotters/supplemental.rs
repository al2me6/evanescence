use std::convert::TryInto;

use evanescence_core::geometry::Plane;
use evanescence_core::orbital::{self, Orbital, RadialPlot};
use wasm_bindgen::JsValue;

use super::isosurface_cutoff_heuristic;
use crate::plotly::color::color_scales;
use crate::plotly::layout::{Axis, Scene, Title};
use crate::plotly::scatter::Line;
use crate::plotly::surface::{Contour, Contours, Lighting, Project};
use crate::plotly::{isosurface, Isosurface, Layout, Scatter, Surface};
use crate::state::State;
use crate::utils::{b16_colors, capitalize_words, partial_max};

pub(crate) fn radial(state: &State) -> (JsValue, JsValue) {
    assert!(state.mode().is_real_or_simple() || state.mode().is_complex());

    let variant: RadialPlot = state.supplement().try_into().unwrap();
    let function_expr = match variant {
        RadialPlot::Wavefunction => "R(r)",
        RadialPlot::ProbabilityDensity => "R(r)²",
        RadialPlot::ProbabilityDistribution => "r²R(r)²",
    };
    let variant_label = format!(
        "{} [ {} ]",
        capitalize_words(state.supplement().to_string()),
        function_expr
    );

    let (x, y) = orbital::sample_radial(state.qn(), variant, state.quality().for_line());

    let trace = Scatter {
        x,
        y,
        line: Line {
            color: Some(b16_colors::BASE[0x0c]),
            ..Default::default()
        },
        ..Default::default()
    };

    let layout = Layout {
        ui_revision: Some(&variant_label),
        drag_mode_bool: Some(false),
        x_axis: Some(Axis {
            title: Some(Title {
                text: "r",
                standoff: Some(20),
            }),
            ..Default::default()
        }),
        y_axis: Some(Axis {
            title: Some(Title {
                text: &variant_label,
                standoff: Some(20),
            }),
            ticks: "outside",
            ..Default::default()
        }),
        ..Default::default()
    };

    (trace.into(), layout.into())
}

pub(crate) fn cross_section(state: &State) -> (JsValue, JsValue) {
    assert!(state.mode().is_real_or_simple() || state.mode().is_hybrid());

    let ui_revision = state.supplement().to_string();
    let plane: Plane = state.supplement().try_into().unwrap();
    let (x_label, y_label) = plane.axes_names();

    let (x, y, mut values) = state.sample_cross_section_real(plane).into_components();

    let max_abs = partial_max(values.iter().flat_map(|row| row.iter()).map(|v| v.abs())).unwrap();

    // If all values are within some very small bound, then it's likely that we have encountered
    // numerical errors and the values should all be zero.
    const ZERO_THRESHOLD: f32 = 1E-7_f32;
    if max_abs < ZERO_THRESHOLD {
        // Zero all values.
        values
            .iter_mut()
            .flat_map(|row| row.iter_mut())
            .for_each(|v| *v = 0.0);
    }

    let contour_max_abs = max_abs * 1.05;

    let trace = Surface {
        x,
        y,
        z: values,
        contours: Some(Contours {
            z: Contour {
                show: Some(true),
                start: Some(-contour_max_abs),
                end: Some(contour_max_abs),
                // For up to 10 contour lines in each polarity, plus one at zero.
                size: Some(contour_max_abs / 11.0),
                use_color_map: Some(true),
                highlight: true,
                project: Some(Project {
                    z: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        }),
        lighting: Some(Lighting::default()),
        ..Default::default()
    };

    let layout = Layout {
        ui_revision: Some(&ui_revision),
        scene: Some(Scene {
            x_axis: Axis {
                title: Some(Title {
                    text: x_label,
                    ..Default::default()
                }),
                ..Default::default()
            },
            y_axis: Axis {
                title: Some(Title {
                    text: y_label,
                    ..Default::default()
                }),
                ..Default::default()
            },
            z_axis: Axis {
                title: Some(Title {
                    text: "Wavefunction Value [ ψ ]",
                    ..Default::default()
                }),
                n_ticks: Some(7),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
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
        lighting: Some(Lighting::default()),
        ..Default::default()
    };

    let layout = Layout {
        ui_revision: Some("isosurface"),
        drag_mode_str: Some("orbit"),
        scene: Some(Scene {
            x_axis: axis,
            y_axis: axis,
            z_axis: axis,
            ..Default::default()
        }),
        ..Default::default()
    };

    (trace.into(), layout.into())
}
