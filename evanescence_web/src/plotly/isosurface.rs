use super::color::ColorScale;
use super::surface::Lighting;
use super::PlotType;

def_plotly_ty! {
    CapsConfig

    show: bool,
}

def_plotly_ty! {
    Caps

    x: CapsConfig,
    y: CapsConfig,
    z: CapsConfig,
}

def_plotly_ty! {
    Surface

    count: u32 = 1,
}

def_plotly_ty! {
    Isosurface<'a>

    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    value: Vec<f32>,
    surface: Surface,
    iso_min as "isomin": f32,
    iso_max as "isomax": f32,
    flat_shading as "flatshading": bool,
    color_scale as "colorscale": ColorScale<'a>,
    opacity: f32 = 1.0,
    show_scale as "showscale": bool,
    caps: Caps,
    plot_type as "type": PlotType = PlotType::Isosurface,
    #optional c_min as "cmin": f32,
    #optional c_max as "cmax": f32,
    lighting: Lighting,
}
