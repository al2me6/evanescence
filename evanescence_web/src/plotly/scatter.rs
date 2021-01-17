use derivative::Derivative;
use serde::Serialize;

use crate::plotly::color::{color_scales, ColorBar, ColorScale};

#[derive(Serialize)]
pub(crate) enum Mode {
    #[serde(rename = "markers")]
    Markers,
}

#[derive(Serialize, Default)]
pub(crate) struct Line {
    width: f32,
}

#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Marker {
    pub(crate) color: Vec<f32>,

    #[serde(rename = "cmid")]
    pub(crate) c_mid: f32,

    #[serde(rename = "colorbar")]
    pub(crate) color_bar: ColorBar,

    #[serde(rename = "colorscale")]
    #[derivative(Default(value = "color_scales::RED_BLUE_REVERSED"))]
    pub(crate) color_scale: ColorScale<'static>,

    pub(crate) line: Line,

    #[derivative(Default(value = "0.98"))]
    pub(crate) opacity: f32,

    #[serde(rename = "showscale")]
    pub(crate) show_scale: bool,
    pub(crate) size: Vec<f32>,
}
