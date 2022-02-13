// Module declarations at end of file.

use serde::Serialize;

pub use self::config::Config;
pub use self::isosurface::Isosurface;
pub use self::layout::{Layout, LayoutRangeUpdate};
pub use self::scatter::Scatter;
pub use self::scatter_3d::Scatter3D;
pub use self::surface::Surface;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PlotType {
    Isosurface,
    #[serde(rename = "scattergl")]
    Scatter,
    Scatter3D,
    Surface,
}

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

        /// Note that we have a `Box<[JsValue]>` and not a `Box<[isize]>`! The latter produces a
        /// `TypedArray` instead of a plain `Array` and causes problems.
        #[wasm_bindgen(js_namespace = Plotly, js_name = deleteTraces)]
        pub fn delete_traces(graph_div: &str, indices: Box<[JsValue]>);

        #[wasm_bindgen(js_namespace = Plotly, js_name = relayout)]
        pub fn relayout(graph_div: &str, update: JsValue);

        #[wasm_bindgen(js_namespace = ["Plotly", "Plots"])]
        pub fn resize(graph_div: &str);
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
/// ```
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
/// ```
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
        $name:ident $(<$($a:lifetime),+>)?
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
        pub struct $name $(<$($a),+>)? {
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

        impl $(<$($a),+>)? ::std::default::Default for $name $(<$($a),+>)? {
            fn default() -> Self {
                Self {
                    $(
                        $($field)? $($optional_field)? : def_plotly_ty!(@field_default $($default)?)
                    ),+
                }
            }
        }

        impl $(<$($a),+>)? ::std::convert::From<$name $(<$($a),+>)?> for wasm_bindgen::JsValue {
            fn from(value: $name) -> Self {
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
pub mod isosurface;
pub mod layout;
pub mod scatter;
pub mod scatter_3d;
pub mod surface;
