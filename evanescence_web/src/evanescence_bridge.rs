use std::convert::TryInto;

use evanescence_core::geometry::{ComponentForm, Plane};
use evanescence_core::monte_carlo::MonteCarlo;
use evanescence_core::numerics::normalize;
use evanescence_core::orbital::{self, wavefunctions, Orbital, RadialPlot};
use wasm_bindgen::JsValue;

use crate::plotly::{
    color::{color_scales, ColorBar, ColorScale},
    layout::{Anchor, Axis, Scene, Title},
    scatter::Line,
    scatter_3d::Marker,
    surface::{Contour, Contours, Lighting, Project},
};
use crate::plotly::{Isosurface, Layout, Scatter, Scatter3D, Surface};
use crate::state::State;
use crate::utils::{capitalize_words, min_max};

pub(crate) fn plot_isosurface(
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

pub(crate) fn plot_pointillist_real(state: &State) -> JsValue {
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

pub(crate) fn plot_radial_nodes(state: &State) -> JsValue {
    plot_isosurface(
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

pub(crate) fn plot_angular_nodes(state: &State) -> JsValue {
    let qn = state.qn;
    plot_isosurface(
        orbital::sample_region_for::<wavefunctions::RealSphericalHarmonic>(
            qn,
            state.quality.for_isosurface(),
            None,
        ),
        qn.l() >= 4 && qn.m().abs() >= 4,
        color_scales::PURP,
    )
}

pub(crate) fn plot_radial(state: &State) -> (JsValue, JsValue) {
    let variant: RadialPlot = state.extra_visualization.try_into().unwrap();
    let variant_name = capitalize_words(&state.extra_visualization.to_string());

    let (x, y) = orbital::Real::sample_radial(state.qn, variant, state.quality.for_line());

    let trace = Scatter {
        x,
        y,
        line: Line {
            color: Some("#8abeb7"),
            ..Default::default()
        },
        ..Default::default()
    };

    let layout = Layout {
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

pub(crate) fn plot_cross_section(state: &State) -> (JsValue, JsValue) {
    let plane: Plane = state.extra_visualization.try_into().unwrap();
    let (x_label, y_label) = plane.axes_names();

    let num_points = state.quality.for_grid();
    let (x, y, mut value) =
        orbital::Real::sample_cross_section(state.qn, plane, num_points).into_components();

    let (min, max) = min_max(value.iter().flat_map(|row| row.iter()));
    let abs_max = max.max(min.abs());

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
        ui_revision: true,
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

pub(crate) fn plot_cross_section_indicator(state: &State) -> JsValue {
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
