pub(crate) mod color;
pub(crate) mod config;
pub(crate) mod isosurface;
pub(crate) mod layout;
pub(crate) mod scatter;
pub(crate) mod scatter_3d;
pub(crate) mod surface;

use serde::Serialize;
use wasm_bindgen::JsValue;

pub(crate) use self::config::Config;
pub(crate) use self::isosurface::Isosurface;
pub(crate) use self::layout::{Layout, LayoutRangeUpdate};
pub(crate) use self::scatter::Scatter;
pub(crate) use self::scatter_3d::Scatter3D;
pub(crate) use self::surface::Surface;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PlotType {
    Isosurface,
    Scatter,
    Scatter3D,
    Surface,
}

#[allow(non_snake_case)] // This is semantically a class.
pub(crate) mod Plotly {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = Plotly, js_name = newPlot)]
        pub(crate) fn new_plot(
            graph_div: &str,
            data: Box<[JsValue]>,
            layout: JsValue,
            config: JsValue,
        );

        #[wasm_bindgen(js_namespace = Plotly, js_name = react)]
        pub(crate) fn react(
            graph_div: &str,
            data: Box<[JsValue]>,
            layout: JsValue,
            config: JsValue,
        );

        #[wasm_bindgen(js_namespace = Plotly, js_name = addTraces)]
        pub(crate) fn add_trace(graph_div: &str, trace: JsValue);

        #[wasm_bindgen(js_namespace = Plotly, js_name = addTraces)]
        pub(crate) fn add_traces(graph_div: &str, trace: Box<[JsValue]>);

        #[wasm_bindgen(js_namespace = Plotly, js_name = deleteTraces)]
        pub(crate) fn delete_trace(graph_div: &str, index: isize);

        /// Note that we have a `Box<[JsValue]>` and not a `Box<[isize]>`! The latter produces a
        /// `TypedArray` instead of a plain `Array` and causes problems.
        #[wasm_bindgen(js_namespace = Plotly, js_name = deleteTraces)]
        pub(crate) fn delete_traces(graph_div: &str, index: Box<[JsValue]>);

        #[wasm_bindgen(js_namespace = Plotly, js_name = relayout)]
        pub(crate) fn relayout(graph_div: &str, update: JsValue);
    }
}

macro_rules! impl_into_js_value {
    ($a:lifetime; $($ty:ident),+) => {$(
        impl<$a> From<$ty<$a>> for JsValue {
            impl_into_js_value!(inner $ty<$a>);
        }
    )+};
    ($($ty:ty),+) => {$(
        impl From<$ty> for JsValue {
            impl_into_js_value!(inner $ty);
        }
    )+};
    (inner $ty:ty) => {
        fn from(value: $ty) -> Self {
            serde_wasm_bindgen::to_value(&value).unwrap()
        }
    }
}
impl_into_js_value!('a; Config, Layout, Isosurface, Scatter, Scatter3D, Surface);
impl_into_js_value!(LayoutRangeUpdate);
