use derivative::Derivative;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::plotly::color::{color_scales, ColorBar, ColorScale};
use crate::plotly::PlotType;
use crate::utils::b16_colors;

#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Project {
    pub(crate) x: bool,
    pub(crate) y: bool,
    pub(crate) z: bool,
}

#[skip_serializing_none]
#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Contour<'a> {
    pub(crate) highlight: bool,
    #[serde(rename = "highlightcolor")]
    #[derivative(Default(value = "b16_colors::BASE[0x0b]"))]
    pub(crate) highlight_color: &'a str,
    pub(crate) start: Option<f32>,
    pub(crate) end: Option<f32>,
    pub(crate) show: Option<bool>,
    pub(crate) size: Option<f32>,
    pub(crate) project: Option<Project>,
    #[serde(rename = "usecolormap")]
    pub(crate) use_color_map: Option<bool>,
}

#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Contours<'a> {
    pub(crate) x: Contour<'a>,
    pub(crate) y: Contour<'a>,
    pub(crate) z: Contour<'a>,
}

#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Lighting {
    pub(crate) diffuse: f32,
    pub(crate) specular: f32,
    pub(crate) roughness: f32,
}

#[skip_serializing_none]
#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Surface<'a> {
    pub(crate) x: Vec<f32>,
    pub(crate) y: Vec<f32>,
    pub(crate) z: Vec<Vec<f32>>,

    #[serde(rename = "cmid")]
    pub(crate) c_mid: f32,

    #[serde(rename = "colorscale")]
    #[derivative(Default(value = "color_scales::RED_BLUE_REVERSED"))]
    pub(crate) color_scale: ColorScale<'static>,

    #[serde(rename = "colorbar")]
    pub(crate) color_bar: ColorBar<'a>,

    #[serde(rename = "surfacecolor")]
    pub(crate) surface_color: Option<Vec<Vec<f32>>>,

    #[serde(rename = "showscale")]
    #[derivative(Default(value = "true"))]
    pub(crate) show_scale: bool,

    #[derivative(Default(value = "1.0"))]
    pub(crate) opacity: f32,

    pub(crate) contours: Option<Contours<'a>>,

    pub(crate) lighting: Option<Lighting>,

    #[serde(rename = "type")]
    #[derivative(Default(value = "PlotType::Surface"))]
    pub(crate) plot_type: PlotType,
}
