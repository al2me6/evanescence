use serde::Serialize;

use super::{Outline, PlotType};

#[derive(Serialize)]
pub enum Mode {
    #[serde(rename = "markers")]
    Markers,
}

def_plotly_ty! {
    Scatter<'a>

    #optional name: &'a str,
    x: Vec<f32>,
    y: Vec<f32>,
    line: Outline<'a>,
    #optional fill: &'a str,
    plot_type as "type": PlotType = PlotType::Scatter,
}
