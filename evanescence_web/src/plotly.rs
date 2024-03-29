use serde::Serialize;

use self::color::{color_scales, ColorBar, ColorScale};
pub use self::config::Config;
pub use self::isosurface::Isosurface;
pub use self::layout::{Layout, LayoutRangeUpdate};
pub use self::scatter::Scatter;
pub use self::scatter_3d::Scatter3D;
pub use self::surface::Surface;

#[allow(non_snake_case)] // This is semantically a class.
pub mod Plotly {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = Plotly, js_name = newPlot)]
        pub fn new_plot(graph_div: &str, data: Box<[JsValue]>, layout: JsValue, config: JsValue);

        #[wasm_bindgen(js_namespace = Plotly, js_name = react)]
        pub fn react(graph_div: &str, data: Box<[JsValue]>, layout: JsValue, config: JsValue);

        #[wasm_bindgen(js_namespace = Plotly, js_name = addTraces)]
        pub fn add_trace(graph_div: &str, trace: JsValue);

        #[wasm_bindgen(js_namespace = Plotly, js_name = addTraces)]
        pub fn add_traces(graph_div: &str, traces: Box<[JsValue]>);

        #[wasm_bindgen(js_namespace = Plotly, js_name = deleteTraces)]
        pub fn delete_trace(graph_div: &str, index: isize);

        #[wasm_bindgen(js_namespace = Plotly, js_name = deleteTraces)]
        fn delete_traces_inner(graph_div: &str, indices: Box<[JsValue]>);

        #[wasm_bindgen(js_namespace = Plotly, js_name = relayout)]
        pub fn relayout(graph_div: &str, update: JsValue);

        #[wasm_bindgen(js_namespace = ["Plotly", "Plots"])]
        pub fn resize(graph_div: &str);
    }

    pub fn delete_traces(graph_div: &str, indices: impl IntoIterator<Item = isize>) {
        delete_traces_inner(
            graph_div,
            indices
                .into_iter()
                .map(|i| JsValue::from_f64(i as _))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
    }
}

/// Define a new Plotly configuration type.
///
/// The [`Default`] and [`serde::Serialize`] traits are automatically derived. Thus, fields must
/// be serializable. Fields must also be `Default` or have an override value provided.
/// provided.
///
/// A field marked `#optional` will have its type wrapped in `[Option]` and be serialized as
/// `#[serde(skip_serializing_if = "Option::is_none")]`.
///
/// To provide (or override) a default value, add `= custom_value` after its type.
///
/// A type can be renamed using the syntax `rust_name as "json_name"`.
///
/// Example:
///
/// ```ignore
/// def_plotly_ty! {
///     #[derive(Clone)] // Additional derives are possible.
///     Config<'a> // Lifetimes can be specified.
///
///     required_config: f32 = 0.0,
///     /// Doc comments are allowed. This config is optional!
///     #optional optional_config: bool,
///     config_with_lifetime as "actual_name": &'a str,
/// }
/// ```
///
/// is transformed into:
///
/// ```ignore
/// #[derive(serde::Serialize, Clone)]
/// pub struct Config<'a> {
///     pub required_config: f32,
///     #[doc = " Doc comments are allowed. This config is optional!"]
///     #[serde(skip_serializing_if = "Option::is_none")]
///     pub optional_config: Option<bool>,
///     #[serde(rename = "actual_name")]
///     pub config_with_lifetime: &'a str,
/// }
///
/// impl<'a> std::default::Default for Config<'a> {
///     fn default() -> Self {
///         Self {
///             required_config: 0.0,
///             optional_config: std::default::Default::default(),
///             config_with_lifetime: std::default::Default::default(),
///         }
///     }
/// }
/// ```
macro_rules! def_plotly_ty {
    (
        $(#[$item_meta:meta])*
        $PlotlyTy:ident $(<$($a:lifetime),+>)?
        $(
            $(#[$field_meta:meta])*
            $($field:ident)?
            $(#optional $optional_field: ident)?
            $(as $rename:literal)?
            : $T:ty
            $(= $default:expr)?
        ),+
        $(,)?
    ) => {
        #[derive(serde::Serialize)]
        $(#[$item_meta])*
        pub struct $PlotlyTy $(<$($a),+>)? {
            $(
                $(#[$field_meta])*
                $(#[serde(rename = $rename)])?
                $(
                    #[serde(skip_serializing_if = "Option::is_none")]
                    pub $optional_field: Option<$T>
                )?
                $(
                    pub $field: $T
                )?
            ),+
        }

        impl $(<$($a),+>)? std::default::Default for $PlotlyTy $(<$($a),+>)? {
            fn default() -> Self {
                Self {
                    $(
                        $($field)? $($optional_field)? : def_plotly_ty!(@field_default $($default)?)
                    ),+
                }
            }
        }

        impl $(<$($a),+>)? std::convert::From<$PlotlyTy $(<$($a),+>)?> for wasm_bindgen::JsValue {
            fn from(value: $PlotlyTy) -> Self {
                serde_wasm_bindgen::to_value(&value).unwrap()
            }
        }
    };

    (@field_default) => {
        std::default::Default::default()
    };

    (@field_default $default:expr) => {
        $default
    };
}

pub mod color;
pub mod config;
pub mod histogram;
pub mod isosurface;
pub mod layout;
pub mod scatter;
pub mod scatter_3d;
pub mod surface;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PlotType {
    Histogram,
    Isosurface,
    #[serde(rename = "scattergl")]
    Scatter,
    Scatter3D,
    Surface,
}

def_plotly_ty! {
    Outline<'a>

    #optional width: f32,
    #optional color: &'a str,
}

def_plotly_ty! {
    Marker<'a>

    #optional color: Vec<f32>,
    #optional color_explicit as "color": &'a str,
    #optional c_min as "cmin": f32,
    c_mid as "cmid": f32,
    #optional c_max as "cmax": f32,
    #optional color_bar as "colorbar": ColorBar<'a>,
    #optional color_scale as "colorscale": ColorScale<'static> = Some(color_scales::RD_BU_R),
    #optional line: Outline<'a> = Some(Outline {
        width: Some(0.0),
        ..Default::default()
    }),
    opacity: f32 = 1.0,
    #optional show_scale as "showscale": bool,
    #optional size: Vec<f32>
}
