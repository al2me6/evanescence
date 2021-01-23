use derivative::Derivative;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Anchor {
    Left,
    Right,
}

#[derive(Clone, Copy, Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Title<'a> {
    pub(crate) text: &'a str,
    #[derivative(Default(value = "20"))]
    pub(crate) standoff: usize,
}

#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Font<'a> {
    #[derivative(Default(value = "\"Lato, sans-serif\""))]
    pub(crate) family: &'a str,
    #[derivative(Default(value = "13"))]
    pub(crate) size: usize,
    #[derivative(Default(value = "\"#d8d8d8\""))]
    pub(crate) color: &'a str,
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

// Colors using Base16 Tomorrow Night.
#[derive(Clone, Copy, Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Axis<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) title: Option<Title<'a>>,

    // Default = "".
    pub(crate) ticks: &'a str,

    #[serde(rename = "tickcolor")]
    // blend of base02 and base03
    #[derivative(Default(value = "\"#676a6c\""))]
    pub(crate) tick_color: &'a str,

    #[serde(rename = "exponentformat")]
    #[derivative(Default(value = "\"power\""))]
    pub(crate) exponent_format: &'a str,

    #[serde(rename = "gridcolor")]
    // blend of base02 and base03
    #[derivative(Default(value = "\"#676a6c\""))]
    pub(crate) grid_color: &'a str,

    #[serde(rename = "zeroline")]
    #[derivative(Default(value = "true"))]
    pub(crate) zero_line: bool,

    #[serde(rename = "zerolinecolor")]
    // base06
    #[derivative(Default(value = "\"#e0e0e0\""))]
    pub(crate) zero_line_color: &'a str,

    #[serde(rename = "showspikes")]
    pub(crate) show_spikes: bool,

    #[serde(rename = "automargin")]
    #[derivative(Default(value = "true"))]
    pub(crate) auto_margin: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(rename = "aspectratio")]
    pub(crate) aspect_ratio: AspectRatio,
    #[serde(rename = "xaxis")]
    pub(crate) x_axis: Axis<'a>,
    #[serde(rename = "yaxis")]
    pub(crate) y_axis: Axis<'a>,
    #[serde(rename = "zaxis")]
    pub(crate) z_axis: Axis<'a>,
}

// Colors using Base16 Tomorrow Night.
#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct ModeBar<'a> {
    #[serde(rename = "bgcolor")]
    // blend of base02 and base03
    #[derivative(Default(value = "\"#676a6c\""))]
    pub(crate) bg_color: &'a str,
    // blend of base03 and base04
    #[derivative(Default(value = "\"#a5a8a5\""))]
    pub(crate) color: &'a str,
    #[serde(rename = "activecolor")]
    // base06
    #[derivative(Default(value = "\"#e0e0e0\""))]
    pub(crate) active_color: &'a str,
}

#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Layout<'a> {
    #[serde(rename = "dragmode")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) drag_mode_str: Option<&'a str>,

    #[serde(rename = "dragmode")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) drag_mode_bool: Option<bool>,

    #[serde(rename = "hovermode")]
    pub(crate) hover_mode: bool,

    pub(crate) margin: Margin,

    /// For 3D plots only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) scene: Option<Scene<'a>>,

    pub(crate) font: Font<'a>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "xaxis")]
    pub(crate) x_axis: Option<Axis<'a>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "yaxis")]
    pub(crate) y_axis: Option<Axis<'a>>,

    #[derivative(Default(value = "\"rgba(0,0,0,0)\""))]
    pub(crate) paper_bgcolor: &'a str,

    #[derivative(Default(value = "\"rgba(0,0,0,0)\""))]
    pub(crate) plot_bgcolor: &'a str,

    #[serde(rename = "modebar")]
    pub(crate) mode_bar: ModeBar<'a>,

    #[serde(rename = "uirevision")]
    pub(crate) ui_revision: bool,
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
