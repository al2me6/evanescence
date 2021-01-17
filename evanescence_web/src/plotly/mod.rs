use derivative::Derivative;
use serde::Serialize;
use serde_wasm_bindgen::to_value as to_js_value;
use wasm_bindgen::prelude::*;

pub(crate) mod color;
pub(crate) mod config;
pub(crate) mod layout;
pub(crate) mod scatter;
pub(crate) mod scatter_3d;

#[derive(Serialize)]
pub(crate) enum PlotType {
    #[serde(rename = "scatter3d")]
    Scatter3D,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ModeBarButtons {
    ResetCameraLastSave3d,
    HoverClosest3d,
}

#[derive(Serialize, Derivative)]
#[derivative(Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Config<'a> {
    #[serde(rename = "displaylogo")]
    pub(crate) display_logo: bool,

    #[derivative(Default(value = "true"))]
    pub(crate) display_mode_bar: bool,

    pub(crate) mode_bar_buttons_to_remove: &'a [ModeBarButtons],

    #[derivative(Default(value = "true"))]
    pub(crate) responsive: bool,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Plotly, js_name = react)]
    fn plotly_react(graph_div: &str, data: Box<[JsValue]>, layout: JsValue, config: JsValue);

    #[wasm_bindgen(js_namespace = Plotly, js_name = deleteTraces)]
    fn plotly_delete_trace(graph_div: &str, index: u32);

    #[wasm_bindgen(js_namespace = Plotly, js_name = addTraces)]
    fn plotly_add_trace(graph_div: &str, trace: JsValue);
}

pub(crate) struct Plotly;

impl Plotly {
    pub(crate) fn react<I>(graph_div: &str, data: I, layout: layout::Layout, config: Config)
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: Serialize,
    {
        plotly_react(
            graph_div,
            data.into_iter()
                .map(|trace| to_js_value(&trace).unwrap())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            to_js_value(&layout).unwrap(),
            to_js_value(&config).unwrap(),
        )
    }

    pub(crate) fn delete_trace(graph_div: &str, index: u32) {
        plotly_delete_trace(graph_div, index)
    }

    pub(crate) fn add_trace<T: Serialize>(graph_div: &str, trace: T) {
        plotly_add_trace(graph_div, to_js_value(&trace).unwrap());
    }
}
