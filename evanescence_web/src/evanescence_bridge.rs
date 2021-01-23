use evanescence_core::geometry::ComponentForm;
use evanescence_core::monte_carlo::{MonteCarlo, Quality};
use evanescence_core::numerics::normalize;
use evanescence_core::orbital::{self, wavefunctions, Orbital, Qn, RadialPlot};
use wasm_bindgen::JsValue;

use crate::plotly::color::{color_scales, ColorBar, ColorScale};
use crate::plotly::layout::{Anchor, Axis, Title};
use crate::plotly::scatter::Line;
use crate::plotly::scatter_3d::Marker;
use crate::plotly::{Isosurface, Layout, Scatter, Scatter3D};
use crate::state::{State, Visualization};
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

pub(crate) fn plot_pointillist_real(qn: Qn, quality: Quality) -> JsValue {
    let simulation = orbital::Real::monte_carlo_simulate(qn, quality);
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

pub(crate) fn plot_radial_nodes(qn: Qn, quality: Quality) -> JsValue {
    plot_isosurface(
        orbital::sample_region_for::<wavefunctions::Radial>(
            qn,
            quality.for_isosurface(),
            // Shrink the extent plotted since radial nodes are found in the central
            // part of the full extent only. This is a heuristic that has been verified
            // to cover all radial nodes from `n` = 2 through 8.
            Some(qn.n() as f32 * 0.05 + 0.125),
        ),
        false,
        color_scales::GREENS,
    )
}

pub(crate) fn plot_angular_nodes(qn: Qn, quality: Quality) -> JsValue {
    plot_isosurface(
        orbital::sample_region_for::<wavefunctions::RealSphericalHarmonic>(
            qn,
            quality.for_isosurface(),
            None,
        ),
        qn.l() >= 4 && qn.m().abs() >= 4,
        color_scales::PURP,
    )
}

pub(crate) fn plot_radial(state: &State) -> (JsValue, JsValue) {
    let variant = match state.extra_visualization {
        Visualization::RadialWavefunction => RadialPlot::Wavefunction,
        Visualization::RadialProbability => RadialPlot::Probability,
        Visualization::RadialProbabilityDistribution => RadialPlot::ProbabilityDistribution,
        _ => unreachable!(),
    };
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
                ..Default::default()
            }),
            ..Default::default()
        }),
        y_axis: Some(Axis {
            title: Some(Title {
                text: &variant_name,
                ..Default::default()
            }),
            ticks: "outside",
            ..Default::default()
        }),
        ..Default::default()
    };

    (trace.into(), layout.into())
}
