use evanescence_core::geometry::ComponentForm;
use evanescence_core::numerics::normalize;

use crate::plotly::color::{color_scales, ColorBar};
use crate::plotly::isosurface::Isosurface;
use crate::plotly::layout::Anchor;
use crate::plotly::scatter_3d::{Marker, Scatter3D};

pub(crate) fn into_scatter3d_real(simulation: ComponentForm<f32>) -> Scatter3D {
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
                .map(|v| normalize(min_abs..=max_abs, 0.2..=2.0, v))
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

pub(crate) fn into_nodal_surface(simulation: ComponentForm<f32>) -> Isosurface {
    let (x, y, z, value) = simulation.into_components();
    Isosurface {
        x,
        y,
        z,
        // HACK: We take the "signed square root", i.e. `sgn(x) * sqrt(|x|)` here to alleviate
        // numerical instability/artifacting by amplifying any deviations from zero. However,
        // this also results in crinkly-looking surfaces.
        value: value
            .into_iter()
            .map(|v| v.signum() * v.abs().sqrt())
            .collect(),
        opacity: 0.075,
        color_scale: color_scales::GREENS,
        ..Default::default()
    }
}
