use std::convert::TryInto;

use evanescence_core::geometry::Plane;
use evanescence_core::orbital::{self, Orbital, RadialPlot};
use wasm_bindgen::JsValue;

use crate::plotly::{
    color::color_scales,
    isosurface,
    layout::{Axis, Scene, Title},
    scatter::Line,
    surface::{Contour, Contours, Lighting, Project},
};
use crate::plotly::{Isosurface, Layout, Scatter, Surface};
use crate::state::State;
use crate::utils::{abs_max, b16_colors, capitalize_words};

pub(crate) fn radial(state: &State) -> (JsValue, JsValue) {
    let variant: RadialPlot = state.extra_visualization.try_into().unwrap();
    let variant_name = capitalize_words(&state.extra_visualization.to_string());

    let (x, y) = orbital::Real::sample_radial(state.qn, variant, state.quality.for_line());

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
        ui_revision: Some(&variant_name),
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
                text: &variant_name,
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
    let ui_revision = state.extra_visualization.to_string();
    let plane: Plane = state.extra_visualization.try_into().unwrap();
    let (x_label, y_label) = plane.axes_names();

    let num_points = state.quality.for_grid();
    let (x, y, mut value) =
        orbital::Real::sample_cross_section(state.qn, plane, num_points).into_components();

    let abs_max = abs_max(value.iter().flat_map(|row| row.iter()));

    // If all values are within some very small bound, then it's likely that we have encountered
    // numerical errors and the values should all be zero.
    const ZERO_THRESHOLD: f32 = 1E-7_f32;
    if abs_max < ZERO_THRESHOLD {
        value = vec![vec![0.0; value[0].len()]; value.len()]; // Grid of zeroes.
    }

    let contour_abs_max = abs_max * 1.05;

    let trace = Surface {
        x,
        y,
        z: value,
        contours: Some(Contours {
            z: Contour {
                show: Some(true),
                start: Some(-contour_abs_max),
                end: Some(contour_abs_max),
                // For up to 10 contour lines in each polarity, plus one at zero.
                size: Some(contour_abs_max / 11.0),
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
        lighting: Some(Lighting {
            diffuse: 0.2,
            specular: 0.05,
            roughness: 1.0,
        }),
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
                    text: "Wavefunction Value",
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
    let (x, y, z, value) = orbital::sample_region_for::<orbital::Real>(
        state.qn,
        state.quality.for_isosurface() * 3 / 2,
        None,
    )
    .into_components();

    // Yet another heuristic for scaling the cutoff value appropriately. As the number of lobes
    // increases, they attain increasingly small values, which require a lower cutoff to achieve
    // an adequate appearance (i.e., not showing too small of a portion).
    let num_radial_nodes = orbital::Real::num_radial_nodes(state.qn);
    let num_angular_nodes = orbital::Real::num_angular_nodes(state.qn);
    let num_lobes = (num_radial_nodes + 1) * (num_angular_nodes + 1);
    let damping_factor = if num_radial_nodes == 0 && num_angular_nodes > 2 {
        0.06
    } else {
        0.012
    };
    let cutoff = 0.003 / ((num_lobes as f32 - 1.0).powf(2.5) * damping_factor + 1.0);

    let axis = Axis::from_range_of(state.qn);
    let trace = Isosurface {
        x,
        y,
        z,
        value,
        iso_min: -cutoff,
        iso_max: cutoff,
        surface: isosurface::Surface { count: 2 },
        color_scale: color_scales::RED_BLUE_REVERSED,
        opacity: if state.qn.l() == 0 { 0.5 } else { 1.0 },
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
