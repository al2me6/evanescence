use super::color::{color_scales, ColorBar, ColorScale};
use super::PlotType;
use crate::utils::b16_colors;

def_plotly_ty! {
    #[derive(Clone)]
    Project

    x: bool,
    y: bool,
    z: bool,
}

def_plotly_ty! {
    #[derive(Clone)]
    Contour<'a>

    #optional color: &'a str,
    highlight: bool,
    highlight_color as "highlightcolor": &'a str = b16_colors::BASE[0x0b],
    #optional start: f32,
    #optional end: f32,
    #optional show: bool,
    #optional size: f32,
    #optional project: Project,
    #optional use_color_map as "usecolormap": bool,
}

def_plotly_ty! {
    Contours<'a>

    x: Contour<'a>,
    y: Contour<'a>,
    z: Contour<'a>,
}

def_plotly_ty! {
    Lighting

    diffuse: f32 = 0.2,
    specular: f32 = 0.05,
    roughness: f32 = 1.0,
}

def_plotly_ty! {
    Surface<'a>

    #optional x: Vec<f32>,
    #optional x_parametric as "x": Vec<Vec<f32>>,
    #optional y: Vec<f32>,
    #optional y_parametric as "y": Vec<Vec<f32>>,
    z: Vec<Vec<f32>>,
    #optional c_min as "cmin": f32,
    c_mid as "cmid": f32,
    #optional c_max as "cmax": f32,
    color_scale as "colorscale": ColorScale<'static> = color_scales::RD_BU_R,
    color_bar as "colorbar": ColorBar<'a>,
    #optional surface_color as "surfacecolor": Vec<Vec<f32>>,
    show_scale as "showscale": bool = true,
    opacity: f32 = 1.0,
    #optional contours: Contours<'a>,
    lighting: Lighting,
    plot_type as "type": PlotType = PlotType::Surface,
}
