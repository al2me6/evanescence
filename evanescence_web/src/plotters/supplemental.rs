use std::convert::TryInto;
use std::default::default;
use std::f32::consts::PI;

use evanescence_core::geometry::Plane;
use evanescence_core::numerics::EvaluateBounded;
use evanescence_core::orbital::{self, Complex, RadialPlot, Real};
use wasm_bindgen::JsValue;

use crate::plotly::color::{self, color_scales, ColorBar};
use crate::plotly::layout::{Axis, Font, Scene, Title};
use crate::plotly::scatter::Line;
use crate::plotly::surface::{Contour, Contours, Project};
use crate::plotly::{isosurface, Isosurface, Layout, Scatter, Surface};
use crate::state::State;
use crate::utils::{self, b16_colors};

const ZERO_THRESHOLD: f32 = 1E-7;

fn zero_values(grid: &mut Vec<Vec<f32>>) {
    grid.iter_mut()
        .flat_map(|row| row.iter_mut())
        .for_each(|v| *v = 0.0);
}

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
        fill: (variant == RadialPlot::ProbabilityDistribution).then(|| "tozeroy"),
        ..default()
    };

    let layout = Layout {
        ui_revision: Some(variant_label.clone()),
        drag_mode_bool: Some(false),
        x_axis: Some(Axis {
            title: Some(Title {
                text: "r (Bohr radii)",
                standoff: Some(20),
                ..default()
            }),
            ticks: "outside",
            ..default()
        }),
        y_axis: Some(Axis {
            title: Some(Title {
                text: &variant_label,
                standoff: Some(20),
                ..default()
            }),
            range_mode: Some("tozero"),
            ticks: "outside",
            ..default()
        }),
        ..default()
    };

    (trace.into(), layout.into())
}

fn cross_section_layout(plane: Plane, z_axis_title: &str) -> Layout<'_> {
    let (x_label, y_label) = plane.axes_names();
    Layout {
        ui_revision: Some(plane.to_string()),
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
                    standoff: Some(20),
                    font: Some(Font {
                        size: 14.5,
                        ..default()
                    }),
                }),
                n_ticks: Some(6),
                ..default()
            },
            ..default()
        }),
        ..default()
    }
}

fn cross_section_z_contour(max_abs: f32) -> Contour<'static> {
    let contour_max_abs = max_abs * 1.05;
    Contour {
        show: Some(true),
        start: Some(-contour_max_abs),
        end: Some(contour_max_abs),
        // For up to 10 contour lines for each sign, plus one at zero.
        size: Some(contour_max_abs / 11.0),
        use_color_map: Some(true),
        highlight: true,
        project: Some(Project {
            z: true,
            ..default()
        }),
        ..Default::default()
    }
}

pub(crate) fn cross_section(state: &State) -> (JsValue, JsValue) {
    let is_complex = state.mode().is_complex();
    let plane: Plane = state.supplement().try_into().unwrap();

    let (x, y, mut z, mut custom_color) = if is_complex {
        let (x, y, values) =
            Complex::sample_plane(state.qn(), plane, state.quality().for_grid()).into_components();
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
        let (x, y, z) = state.sample_plane_real(plane).into_components();
        (x, y, z, None)
    };

    let max_abs = utils::partial_max(z.iter().flat_map(|row| row.iter()).map(|v| v.abs())).unwrap();

    if max_abs < ZERO_THRESHOLD {
        zero_values(&mut z);
        if let Some(ref mut color) = custom_color {
            zero_values(color);
        }
    }

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
            color_scales::RD_BU_R
        },
        c_min: is_complex.then(|| -PI),
        c_mid: 0.0,
        c_max: is_complex.then(|| PI),
        contours: Some(Contours {
            z: Contour {
                color: is_complex.then(|| b16_colors::BASE[0x03]), // Use default otherwise.
                use_color_map: Some(!is_complex),
                ..cross_section_z_contour(max_abs)
            },
            ..default()
        }),
        ..default()
    };

    let z_axis_title = if is_complex {
        format!("Wavefunction Modulus [ |ψ{}| ]", plane.ordered_triple())
    } else {
        format!("Wavefunction [ ψ{} ]", plane.ordered_triple())
    };
    let layout = cross_section_layout(plane, &z_axis_title);

    (trace.into(), layout.into())
}

pub(crate) fn cross_section_prob_density(state: &State) -> (JsValue, JsValue) {
    let plane: Plane = state.supplement().try_into().unwrap();
    let (x, y, mut z) = state.sample_plane_prob_density(plane).into_components();
    let max = *utils::partial_max(z.iter().flat_map(|row| row.iter())).unwrap();
    assert!(
        max >= 0.0,
        "probability densities must be positive; got {}",
        max
    );

    if max < ZERO_THRESHOLD {
        zero_values(&mut z);
    }

    let z_contour = cross_section_z_contour(max);
    let trace = Surface {
        x,
        y,
        z,
        c_min: Some(0.0),
        c_max: Some(if max < ZERO_THRESHOLD { 1.0 } else { max }),
        color_scale: color_scales::TEMPO,
        contours: Some(Contours {
            z: Contour {
                start: Some(max * 0.005),
                size: z_contour.size.map(|s| s / 2.0),
                ..z_contour
            },
            ..default()
        }),
        ..default()
    };

    let z_axis_title = format!("Prob. Density [ |ψ{}|² ]", plane.ordered_triple());
    let layout = cross_section_layout(plane, &z_axis_title);

    (trace.into(), layout.into())
}

pub(crate) fn isosurface_3d(state: &State) -> (JsValue, JsValue) {
    assert!(state.mode().is_real_or_simple() || state.mode().is_hybrid());

    let trace = if state.mode().is_hybrid() {
        super::compute_isosurface_hybrid(state.hybrid_kind(), 0, state.quality())
    } else {
        let (x, y, z, value) =
            Real::sample_region(state.qn(), state.quality().for_isosurface() * 3 / 2)
                .into_components();
        let cutoff = super::isosurface_cutoff_heuristic_real(state.qn());
        Isosurface {
            x,
            y,
            z,
            value,
            iso_min: -cutoff,
            iso_max: cutoff,
            surface: isosurface::Surface { count: 2 },
            color_scale: color_scales::RD_BU_R,
            opacity: if state.qn().l() == 0 { 0.5 } else { 1.0 },
            ..default()
        }
    };
    let cutoff = trace.iso_max;
    let trace = Isosurface {
        c_min: Some(-cutoff * 1.2),
        c_max: Some(cutoff * 1.2),
        ..trace
    };

    let axis = Axis::with_extent(state.bound());
    let layout = Layout {
        ui_revision: Some("isosurface".to_owned()),
        drag_mode_str: Some("orbit"),
        scene: Some(Scene {
            x_axis: axis.clone(),
            y_axis: axis.clone(),
            z_axis: axis.clone(),
            ..default()
        }),
        ..default()
    };

    (trace.into(), layout.into())
}
