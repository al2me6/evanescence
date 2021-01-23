use derivative::Derivative;
use serde::Serialize;

use crate::plotly::color::{color_scales, ColorBar, ColorScale};
use crate::plotly::PlotType;

#[derive(Serialize)]
pub(crate) enum Mode {
    #[serde(rename = "markers")]
    Markers,
}

#[derive(Serialize, Default)]
pub(crate) struct Line<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) color: Option<&'a str>,
}

#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Marker<'a> {
    pub(crate) color: Vec<f32>,

    #[serde(rename = "cmid")]
    pub(crate) c_mid: f32,

    #[serde(rename = "colorbar")]
    pub(crate) color_bar: ColorBar<'a>,

    #[serde(rename = "colorscale")]
    #[derivative(Default(value = "color_scales::RED_BLUE_REVERSED"))]
    pub(crate) color_scale: ColorScale<'static>,

    #[derivative(Default(value = "Line { width: Some(0.0), ..Default::default() }"))]
    pub(crate) line: Line<'a>,

    #[derivative(Default(value = "0.98"))]
    pub(crate) opacity: f32,

    #[serde(rename = "showscale")]
    pub(crate) show_scale: bool,
    pub(crate) size: Vec<f32>,
}

#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Scatter<'a> {
    pub(crate) x: Vec<f32>,
    pub(crate) y: Vec<f32>,
    pub(crate) line: Line<'a>,
    #[derivative(Default(value = "PlotType::Scatter"))]
    pub(crate) plot_type: PlotType,
}
