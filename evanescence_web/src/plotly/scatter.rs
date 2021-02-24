use serde::Serialize;

use super::color::{color_scales, ColorBar, ColorScale};
use super::PlotType;

#[derive(Serialize)]
pub(crate) enum Mode {
    #[serde(rename = "markers")]
    Markers,
}

def_plotly_ty! {
    Line<'a>

    #optional width: f32,
    #optional color: &'a str,
}

def_plotly_ty! {
    Marker<'a>

    color: Vec<f32>,
    #optional c_min as "cmin": f32,
    c_mid as "cmid": f32,
    #optional c_max as "cmax": f32,
    color_bar as "colorbar": ColorBar<'a>,
    color_scale as "colorscale": ColorScale<'static> = color_scales::RED_BLUE_REVERSED,
    line: Line<'a> = Line {
        width: Some(0.0),
        ..Default::default()
    },
    opacity: f32 = 1.0,
    show_scale as "showscale": bool,
    size: Vec<f32>
}

def_plotly_ty! {
    Scatter<'a>

    x: Vec<f32>,
    y: Vec<f32>,
    line: Line<'a>,
    plot_type as "type": PlotType = PlotType::Scatter,
}
