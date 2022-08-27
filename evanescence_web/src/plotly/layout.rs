use serde::Serialize;

use crate::utils::b16_colors;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Anchor {
    Auto,
    Top,
    Right,
    Bottom,
    Left,
    Center,
}

def_plotly_ty! {
    #[derive(Clone, Copy)]
    Font<'a>

    family: &'a str = "Lato, sans-serif",
    size: f32 = 13.0,
    color: &'a str = b16_colors::BASE[0x05],
}

def_plotly_ty! {
    #[derive(Clone)]
    Title<'a>

    text: &'a str,
    #optional font: Font<'a>,
    #optional standoff: usize,
}

def_plotly_ty! {
    Margin

    top    as "t": u32,
    right  as "r": u32,
    bottom as "b": u32,
    left   as "l": u32,
}

def_plotly_ty! {
    #[derive(Clone)]
    Axis<'a>

    #optional title: Title<'a>,
    #optional range_mode as "rangemode": &'a str,
    ticks: &'a str,
    #optional n_ticks as "nticks": u32,
    tick_length as "ticklen": u32 = 6,
    tick_color as "tickcolor": &'a str = b16_colors::BASE0203,
    exponent_format as "exponentformat": &'a str = "power",
    grid_color as "gridcolor": &'a str = b16_colors::BASE0203,
    zero_line as "zeroline": bool = true,
    zero_line_color as "zerolinecolor": &'a str = b16_colors::BASE[0x06],
    show_spikes as "showspikes": bool,
    auto_margin as "automargin": bool = true,
    #optional range: (f32, f32),
}

impl<'a> Axis<'a> {
    pub fn with_extent(extent: f32) -> Self {
        Self {
            range: Some((-extent, extent)),
            ..Default::default()
        }
    }
}

def_plotly_ty! {
    Legend

    #optional x: f32,
    #optional y: f32,
    #optional x_anchor as "xanchor": Anchor,
    #optional y_anchor as "yanchor": Anchor,
}

def_plotly_ty! {
    AspectRatio

    x: f32 = 1.0,
    y: f32 = 1.0,
    z: f32 = 1.0,
}

def_plotly_ty! {
    Scene<'a>

    aspect_mode as "aspectmode": &'a str = "manual",
    aspect_ratio as "aspectratio": AspectRatio,
    x_axis as "xaxis": Axis<'a>,
    y_axis as "yaxis": Axis<'a>,
    z_axis as "zaxis": Axis<'a>,
}

def_plotly_ty! {
    ModeBar<'a>

    bg_color as "bgcolor": &'a str = b16_colors::BASE0203,
    color: &'a str = b16_colors::BASE0304,
    active_color as "activecolor": &'a str = b16_colors::BASE[0x06],
}

def_plotly_ty! {
    Layout<'a>

    #optional drag_mode_str as "dragmode": &'a str,
    #optional drag_mode_bool as "dragmode": bool,
    #optional hover_mode_bool as "hovermode": bool = Some(false),
    #optional legend: Legend,
    margin: Margin,
    /// For 3D plots only.
    #optional scene: Scene<'a>,
    font: Font<'a>,
    /// For 2D plots only.
    #optional x_axis as "xaxis": Axis<'a>,
    /// For 2D plots only.
    #optional y_axis as "yaxis": Axis<'a>,
    paper_bgcolor: &'a str = b16_colors::BASE0102,
    plot_bgcolor: &'a str = b16_colors::BASE0102,
    mode_bar as "modebar": ModeBar<'a>,
    #optional ui_revision as "uirevision": String,
}

def_plotly_ty! {
    LayoutRangeUpdate

    x_axis_range as "scene.xaxis.range": (f32, f32),
    y_axis_range as "scene.yaxis.range": (f32, f32),
    z_axis_range as "scene.zaxis.range": (f32, f32),
}

impl LayoutRangeUpdate {
    pub fn new(extent: f32) -> Self {
        let range = (-extent, extent);
        Self {
            x_axis_range: range,
            y_axis_range: range,
            z_axis_range: range,
        }
    }
}
