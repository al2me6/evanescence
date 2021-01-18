use derivative::Derivative;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Anchor {
    Left,
    Right,
}

#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Margin {
    #[serde(rename = "t")]
    pub(crate) top: u32,
    #[serde(rename = "r")]
    pub(crate) right: u32,
    #[serde(rename = "b")]
    pub(crate) bottom: u32,
    #[serde(rename = "l")]
    pub(crate) left: u32,
}

#[derive(Clone, Copy, Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Axis<'a> {
    #[serde(rename = "backgroundcolor")]
    #[derivative(Default(value = "\"rgb(40,40,40)\""))]
    pub(crate) background_color: &'a str,

    #[serde(rename = "gridcolor")]
    #[derivative(Default(value = "\"rgb(140,140,140)\""))]
    pub(crate) grid_color: &'a str,

    #[serde(rename = "zerolinecolor")]
    #[derivative(Default(value = "\"rgb(240,240,240)\""))]
    pub(crate) zero_line_color: &'a str,

    #[serde(rename = "showbackground")]
    #[derivative(Default(value = "true"))]
    pub(crate) show_background: bool,

    #[serde(rename = "showspikes")]
    pub(crate) show_spikes: bool,

    pub(crate) range: Option<(f32, f32)>,
}

#[derive(Serialize)]
pub(crate) struct AspectRatio {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
}

impl Default for AspectRatio {
    fn default() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }
}

#[derive(Serialize, Derivative)]
#[derivative(Default())]
pub(crate) struct Scene<'a> {
    #[serde(rename = "aspectmode")]
    #[derivative(Default(value = "\"manual\""))]
    pub(crate) aspect_mode: &'a str,
    #[serde(rename="aspectratio")]
    pub(crate) aspect_ratio: AspectRatio,
    #[serde(rename = "xaxis")]
    pub(crate) x_axis: Axis<'a>,
    #[serde(rename = "yaxis")]
    pub(crate) y_axis: Axis<'a>,
    #[serde(rename = "zaxis")]
    pub(crate) z_axis: Axis<'a>,
}

#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Layout<'a> {
    #[serde(rename = "dragmode")]
    #[derivative(Default(value = "\"orbit\""))]
    pub(crate) drag_mode: &'a str,

    #[serde(rename = "hovermode")]
    pub(crate) hover_mode: bool,

    pub(crate) margin: Margin,
    pub(crate) scene: Option<Scene<'a>>,

    #[serde(rename = "uirevision")]
    pub(crate) ui_revision: &'a str,
}

#[derive(Serialize)]
pub(crate) struct LayoutRangeUpdate {
    #[serde(rename = "scene.xaxis.range")]
    pub(crate) x_axis_range: (f32, f32),
    #[serde(rename = "scene.yaxis.range")]
    pub(crate) y_axis_range: (f32, f32),
    #[serde(rename = "scene.zaxis.range")]
    pub(crate) z_axis_range: (f32, f32),
}

impl LayoutRangeUpdate {
    pub(crate) fn new(extent: f32) -> Self {
        let range = (-extent, extent);
        Self {
            x_axis_range: range,
            y_axis_range: range,
            z_axis_range: range,
        }
    }
}
