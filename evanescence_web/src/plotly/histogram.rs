use super::{Marker, PlotType};

def_plotly_ty! {
    Bins

    start: f32,
    #optional size: f32,
    end: f32,
}

def_plotly_ty! {
    Histogram<'a>

    #optional name: &'a str,
    x: Vec<f32>,
    #optional hist_norm as "histnorm": &'a str,
    #optional x_bins as "xbins": Bins,
    marker: Marker<'a>,
    #optional opacity: f32,
    plot_type as "type": PlotType = PlotType::Histogram,
}
