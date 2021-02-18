use derivative::Derivative;
use serde::Serialize;
use serde_with::skip_serializing_none;

use super::color::{color_scales, ColorBar, ColorScale};
use super::PlotType;

#[derive(Serialize)]
pub(crate) enum Mode {
    #[serde(rename = "markers")]
    Markers,
}

#[skip_serializing_none]
#[derive(Serialize, Default)]
pub(crate) struct Line<'a> {
    pub(crate) width: Option<f32>,
    pub(crate) color: Option<&'a str>,
}

#[skip_serializing_none]
#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Marker<'a> {
    pub(crate) color: Vec<f32>,

    #[serde(rename = "cmin")]
    pub(crate) c_min: Option<f32>,

    #[serde(rename = "cmid")]
    pub(crate) c_mid: f32,

    #[serde(rename = "cmax")]
    pub(crate) c_max: Option<f32>,

    #[serde(rename = "colorbar")]
    pub(crate) color_bar: ColorBar<'a>,

    #[serde(rename = "colorscale")]
    #[derivative(Default(value = "color_scales::RED_BLUE_REVERSED"))]
    pub(crate) color_scale: ColorScale<'static>,

    #[derivative(Default(value = "Line { width: Some(0.0), ..Default::default() }"))]
    pub(crate) line: Line<'a>,

    #[derivative(Default(value = "1.0"))]
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
