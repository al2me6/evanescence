pub(crate) mod color;
pub(crate) mod config;
pub(crate) mod isosurface;
pub(crate) mod layout;
pub(crate) mod scatter;
pub(crate) mod scatter_3d;

use serde::Serialize;
use serde_wasm_bindgen::to_value as to_js_value;
use wasm_bindgen::prelude::*;

use self::config::Config;
use self::layout::Layout;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PlotType {
    Scatter3D,
    Isosurface,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Plotly, js_name = react)]
    fn plotly_react(graph_div: &str, data: Box<[JsValue]>, layout: JsValue, config: JsValue);

    #[wasm_bindgen(js_namespace = Plotly, js_name = deleteTraces)]
    fn plotly_delete_trace(graph_div: &str, index: isize);

    #[wasm_bindgen(js_namespace = Plotly, js_name = addTraces)]
    fn plotly_add_trace(graph_div: &str, trace: JsValue);

    #[wasm_bindgen(js_namespace = Plotly, js_name = addTraces)]
    fn plotly_add_trace_at(graph_div: &str, trace: JsValue, index: isize);

    #[wasm_bindgen(js_namespace = Plotly, js_name = relayout)]
    fn plotly_relayout(graph_div: &str, update: JsValue);
}

pub(crate) struct Plotly;

#[allow(dead_code)]
impl Plotly {
    pub(crate) fn react<I>(graph_div: &str, data: I, layout: Layout, config: Config)
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

    pub(crate) fn delete_trace(graph_div: &str, index: isize) {
        plotly_delete_trace(graph_div, index)
    }

    pub(crate) fn add_trace<T: Serialize>(graph_div: &str, trace: T) {
        plotly_add_trace(graph_div, to_js_value(&trace).unwrap());
    }

    pub(crate) fn add_trace_at<T: Serialize>(graph_div: &str, trace: T, index: isize) {
        plotly_add_trace_at(graph_div, to_js_value(&trace).unwrap(), index);
    }

    pub(crate) fn relayout<T: Serialize>(graph_div: &str, layout: T) {
        plotly_relayout(graph_div, to_js_value(&layout).unwrap());
    }
}
