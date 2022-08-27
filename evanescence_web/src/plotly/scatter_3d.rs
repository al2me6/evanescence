pub use super::scatter::Mode;
use super::{Marker, PlotType};

def_plotly_ty! {
    Scatter3D<'a>

    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    mode: Mode = Mode::Markers,
    marker: Marker<'a>,
    show_legend as "showlegend": bool = true,
    plot_type as "type": PlotType = PlotType::Scatter3D,
}
