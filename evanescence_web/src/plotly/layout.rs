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

#[derive(Serialize, Derivative)]
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
}

#[derive(Serialize, Derivative)]
#[derivative(Default(new = "true"))]
pub(crate) struct Scene<'a> {
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
