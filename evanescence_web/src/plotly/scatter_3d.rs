pub(crate) use super::scatter::{Marker, Mode};
use super::PlotType;

def_plotly_ty! {
    Scatter3D<'a>

    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    mode: Mode = Mode::Markers,
    marker: Marker<'a>,
    plot_type as "type": PlotType = PlotType::Scatter3D,
}
