use evanescence_core::geometry::ComponentForm;
use evanescence_core::monte_carlo::Quality;
use evanescence_core::numerics::normalize;
use evanescence_core::orbital::{self, wavefunctions, Qn};

use crate::plotly::color::{color_scales, ColorBar, ColorScale};
use crate::plotly::isosurface::Isosurface;
use crate::plotly::layout::Anchor;
use crate::plotly::scatter_3d::{Marker, Scatter3D};

pub(crate) fn plot_scatter3d_real(simulation: ComponentForm<f32>) -> Scatter3D {
    let (x, y, z, values) = simulation.into_components();

    let values_abs: Vec<_> = values.iter().map(|&v| v.abs()).collect();
    let (min_abs, max_abs) = values_abs
        .iter()
        .fold((0.0_f32, 0.0_f32), |(curr_min, curr_max), &v| {
            (curr_min.min(v), curr_max.max(v))
        });

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
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

pub(crate) fn plot_isosurface(
    simulation: ComponentForm<f32>,
    correct_instability: bool,
    color_scale: ColorScale,
) -> Isosurface<'_> {
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
}

pub(crate) fn plot_radial_nodes(qn: Qn, quality: Quality) -> Isosurface<'static> {
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

pub(crate) fn plot_angular_nodes(qn: Qn, quality: Quality) -> Isosurface<'static> {
    plot_isosurface(
        orbital::sample_region_for::<wavefunctions::RealSphericalHarmonic>(
            qn,
            quality.for_isosurface(),
            None,
        ),
        qn.l() > 6 && qn.m().abs() > 5,
        color_scales::PURP,
    )
}
