use std::default::default;
use std::f32::consts::PI;

use evanescence_core::geometry::storage::grid_values::CoordinatePlane3;
use evanescence_core::numerics::function::Function3InOriginCenteredRegionExt;
use evanescence_core::numerics::statistics::Pdf;
use evanescence_core::numerics::Function;
use evanescence_core::orbital::atomic::{Radial, RadialProbabilityDistribution};
use evanescence_core::orbital::quantum_numbers::Nl;
use evanescence_core::orbital::{AtomicComplex, AtomicReal};
use na::vector;
use wasm_bindgen::JsValue;

use crate::plotly::color::{self, color_scales, ColorBar};
use crate::plotly::histogram::{Bins, Histogram};
use crate::plotly::layout::{Anchor, Axis, Font, Legend, Scene, Title};
use crate::plotly::surface::{Contour, Contours, Project};
use crate::plotly::{isosurface, Isosurface, Layout, Marker, Outline, Scatter, Surface};
use crate::state::cache::MONTE_CARLO_CACHE;
use crate::state::{Mode, State, Visualization};
use crate::utils::{self, b16_colors};

type TraceLayout = (Vec<JsValue>, JsValue);

const ZERO_THRESHOLD: f32 = 1E-7;

#[allow(clippy::ptr_arg)] // This is a function intended to be used with a specific type.
fn zero_values(grid: &mut Vec<Vec<f32>>) {
    grid.iter_mut()
        .flat_map(|row| row.iter_mut())
        .for_each(|v| *v = 0.0);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RadialPlot {
    Wavefunction,
    ProbabilityDistribution,
    CumulativeDistribution,
}

impl RadialPlot {
    pub fn sample(self, nl: Nl, extent: f32, num_points: usize) -> ([Vec<f32>; 1], Vec<f32>) {
        let evaluator: Box<dyn Function<1, _, Output = _>> = match self {
            Self::Wavefunction => Box::new(Radial::new(nl)),
            Self::ProbabilityDistribution => Box::new(RadialProbabilityDistribution::new(nl)),
            Self::CumulativeDistribution => {
                return Pdf::new(RadialProbabilityDistribution::new(nl))
                    .sample_cdf(0.0..=extent, num_points)
                    .into_components();
            }
        };
        evaluator
            .sample_from_line_segment(vector![0.]..=vector![extent], num_points)
            .into_components()
    }
}

#[allow(clippy::too_many_lines)]
pub fn radial(state: &State) -> TraceLayout {
    const NUM_POINTS: usize = 600;
    const HIST_BINS: usize = 100;

    assert!(state.mode().is_real_or_simple() || state.mode().is_complex());

    let variant: RadialPlot = match state.supplement() {
        Visualization::RadialWavefunction => RadialPlot::Wavefunction,
        Visualization::RadialProbabilityDistribution => RadialPlot::ProbabilityDistribution,
        Visualization::CumulativeRadialDistribution => RadialPlot::CumulativeDistribution,
        _ => panic!("unexpected visualization type"),
    };
    let function_expr = match variant {
        RadialPlot::Wavefunction => "R(r)",
        RadialPlot::ProbabilityDistribution => "r²R(r)²",
        // FIXME: Unicode scripts look terrible.
        RadialPlot::CumulativeDistribution => "∫₀ʳ s²R(s)² ds",
    };
    let variant_label = format!(
        "{} [ {function_expr} ]",
        utils::capitalize_words(state.supplement().to_string())
    );

    let bound = state.bound();
    let ([x], y) = variant.sample(state.qn().into(), bound, NUM_POINTS);

    let curve = Scatter {
        name: Some("Theoretical"),
        x,
        y,
        line: Outline {
            color: Some(
                b16_colors::BASE[match variant {
                    RadialPlot::Wavefunction => 0x0c,
                    RadialPlot::ProbabilityDistribution => 0x0a,
                    RadialPlot::CumulativeDistribution => 0x0b,
                }],
            ),
            ..default()
        },
        fill: (variant == RadialPlot::ProbabilityDistribution).then_some("tozeroy"),
        ..default()
    };

    let traces = if variant == RadialPlot::ProbabilityDistribution {
        let mut cache = MONTE_CARLO_CACHE.lock().unwrap();
        let [xs, ys, zs] = if state.mode().is_complex() {
            cache
                .request_complex32(state.into(), state.quality().point_cloud())
                .unwrap()
                .coords()
        } else {
            cache
                .request_f32(state.into(), state.quality().point_cloud())
                .unwrap()
                .coords()
        };
        let rs = itertools::izip!(xs, ys, zs)
            .map(|(&x, &y, &z)| vector![x, y, z].norm())
            .collect();

        let bin_size = bound / (HIST_BINS as f32);

        let histogram = Histogram {
            name: Some("Sampled"),
            x: rs,
            hist_norm: Some("probability density"),
            x_bins: Some(Bins {
                start: 0.0,
                size: Some(bin_size),
                end: bound * 0.99,
            }),
            marker: Marker {
                color_explicit: Some("rgba(253,141,60,0.75)"),
                line: Some(Outline {
                    width: Some(1.0),
                    color: Some(color_scales::ORANGES[4].1),
                }),
                ..default()
            },
            opacity: Some(0.75),
            ..default()
        };

        vec![curve.into(), histogram.into()]
    } else {
        vec![curve.into()]
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
        legend: Some(Legend {
            x: Some(1.0),
            x_anchor: Some(Anchor::Right),
            y: Some(0.925),
            y_anchor: Some(Anchor::Center),
        }),
        ..default()
    };

    (traces, layout.into())
}

fn cross_section_layout(plane: CoordinatePlane3, z_axis_title: &str) -> Layout<'_> {
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
                    text: z_axis_title,
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
        ..default()
    }
}

pub fn cross_section(state: &State) -> TraceLayout {
    let is_complex = state.mode().is_complex();
    let plane: CoordinatePlane3 = state.supplement().try_into().unwrap();

    let (x, y, mut z, mut custom_color) = if is_complex {
        let (x, y, values) = AtomicComplex::new(*state.qn())
            .bounded_sample_in_plane(plane, state.quality().grid_2d())
            .grid_values
            .into_components();
        let (moduli, arguments) = values
            .iter()
            .map(|row| utils::split_moduli_arguments(row))
            .unzip();
        (x, y, moduli, Some(arguments))
    } else {
        let (x, y, z) = state.sample_plane_real(plane).grid_values.into_components();
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
        x: Some(x),
        y: Some(y),
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
        c_min: is_complex.then_some(-PI),
        c_mid: 0.0,
        c_max: is_complex.then_some(PI),
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
        format!(
            "Wavefunction Modulus [ |ψ{}| ]",
            plane.ordered_triple_form()
        )
    } else {
        format!("Wavefunction [ ψ{} ]", plane.ordered_triple_form())
    };
    let layout = cross_section_layout(plane, &z_axis_title);

    (vec![trace.into()], layout.into())
}

pub fn cross_section_prob_density(state: &State) -> TraceLayout {
    let plane: CoordinatePlane3 = state.supplement().try_into().unwrap();
    let (x, y, mut z) = state
        .sample_plane_prob_density(plane)
        .grid_values
        .into_components();
    let max = *utils::partial_max(z.iter().flat_map(|row| row.iter())).unwrap();
    assert!(
        max >= 0.0,
        "probability densities must be positive; got {max}"
    );

    if max < ZERO_THRESHOLD {
        zero_values(&mut z);
    }

    let z_contour = cross_section_z_contour(max);
    let trace = Surface {
        x: Some(x),
        y: Some(y),
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

    let z_axis_title = format!("Prob. Density [ |ψ{}|² ]", plane.ordered_triple_form());
    let layout = cross_section_layout(plane, &z_axis_title);

    (vec![trace.into()], layout.into())
}

pub fn isosurface_3d(state: &State) -> TraceLayout {
    let trace = match state.mode() {
        Mode::RealSimple | Mode::RealFull => {
            let ([x, y, z], value) = AtomicReal::new(*state.qn())
                .bounded_sample_in_cube(state.quality().grid_3d() * 3 / 2)
                .into_components();
            let cutoff = super::isosurface_cutoff_atomic_real(state.qn());
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
        }
        Mode::Hybrid => super::compute_isosurface_hybrid(state.hybrid_kind(), 0, state.quality()),
        Mode::Complex => unreachable!(),
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

    (vec![trace.into()], layout.into())
}
