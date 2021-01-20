use derivative::Derivative;
use serde::Serialize;

pub(crate) use crate::plotly::scatter::{Marker, Mode};
use crate::plotly::PlotType;

#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct Scatter3D<'a> {
    pub(crate) x: Vec<f32>,
    pub(crate) y: Vec<f32>,
    pub(crate) z: Vec<f32>,

    #[derivative(Default(value = "Mode::Markers"))]
    pub(crate) mode: Mode,

    pub(crate) marker: Marker<'a>,

    #[serde(rename = "type")]
    #[derivative(Default(value = "PlotType::Scatter3D"))]
    pub(crate) plot_type: PlotType,
}
