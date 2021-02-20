use derivative::Derivative;
use serde::Serialize;
use serde_with::skip_serializing_none;

use super::color::ColorScale;
use super::surface::Lighting;
use super::PlotType;

#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct CapsConfig {
    show: bool,
}

#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Caps {
    x: CapsConfig,
    y: CapsConfig,
    z: CapsConfig,
}

#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Surface {
    #[derivative(Default(value = "1"))]
    pub(crate) count: u32,
}

#[skip_serializing_none]
#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Isosurface<'a> {
    pub(crate) x: Vec<f32>,
    pub(crate) y: Vec<f32>,
    pub(crate) z: Vec<f32>,
    pub(crate) value: Vec<f32>,
    pub(crate) surface: Surface,
    #[serde(rename = "isomin")]
    pub(crate) iso_min: f32,
    #[serde(rename = "isomax")]
    pub(crate) iso_max: f32,
    #[serde(rename = "flatshading")]
    pub(crate) flat_shading: bool,
    #[serde(rename = "colorscale")]
    pub(crate) color_scale: ColorScale<'a>,
    #[derivative(Default(value = "1.0"))]
    pub(crate) opacity: f32,
    #[serde(rename = "showscale")]
    pub(crate) show_scale: bool,
    pub(crate) caps: Caps,
    #[serde(rename = "type")]
    #[derivative(Default(value = "PlotType::Isosurface"))]
    pub(crate) plot_type: PlotType,
    #[serde(rename = "cmin")]
    pub(crate) c_min: Option<f32>,
    #[serde(rename = "cmax")]
    pub(crate) c_max: Option<f32>,
    pub(crate) lighting: Option<Lighting>,
}
