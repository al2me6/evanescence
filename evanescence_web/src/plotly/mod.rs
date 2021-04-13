// Module declarations at end of file.

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
    #[serde(rename = "scattergl")]
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
        pub(crate) fn add_traces(graph_div: &str, traces: Box<[JsValue]>);

        #[wasm_bindgen(js_namespace = Plotly, js_name = deleteTraces)]
        pub(crate) fn delete_trace(graph_div: &str, index: isize);

        /// Note that we have a `Box<[JsValue]>` and not a `Box<[isize]>`! The latter produces a
        /// `TypedArray` instead of a plain `Array` and causes problems.
        #[wasm_bindgen(js_namespace = Plotly, js_name = deleteTraces)]
        pub(crate) fn delete_traces(graph_div: &str, indices: Box<[JsValue]>);

        #[wasm_bindgen(js_namespace = Plotly, js_name = relayout)]
        pub(crate) fn relayout(graph_div: &str, update: JsValue);
    }
}

macro_rules! impl_into_js_value {
    ($a:lifetime; $($ty:ident),+) => {$(
        impl<$a> From<$ty<$a>> for JsValue {
            impl_into_js_value!(@inner $ty<$a>);
        }
    )+};
    ($($ty:ty),+) => {$(
        impl From<$ty> for JsValue {
            impl_into_js_value!(@inner $ty);
        }
    )+};
    (@inner $ty:ty) => {
        fn from(value: $ty) -> Self {
            serde_wasm_bindgen::to_value(&value).unwrap()
        }
    }
}
impl_into_js_value!('a; Config, Layout, Isosurface, Scatter, Scatter3D, Surface);
impl_into_js_value!(LayoutRangeUpdate);

// Note that the macro must be defined before the modules using it!!

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
/// pub(crate) struct Config<'a> {
///     pub(crate) required_config: f32,
///     #[doc = " Doc comments are allowed. This config is optional!"]
///     #[serde(skip_serializing_if = "Option::is_none")]
///     pub(crate) optional_config: Option<bool>,
///     #[serde(rename = "actual_name")]
///     pub(crate) config_with_lifetime: &'a str,
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
        pub(crate) struct $name $(<$($a),+>)? {
            $(
                $(#[$field_meta])*
                $(#[serde(rename = $rename)])?
                $(
                    #[serde(skip_serializing_if = "Option::is_none")]
                    pub(crate) $optional_field: Option<$T>
                )?
                $(
                    pub(crate) $field: $T
                )?
            ),+
        }

        impl $(<$($a),+>)? std::default::Default for $name $(<$($a),+>)? {
            fn default() -> Self {
                Self {
                    $(
                        $($field)? $($optional_field)? : def_plotly_ty!(@field_default $($default)?)
                    ),+
                }
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

pub(crate) mod color;
pub(crate) mod config;
pub(crate) mod isosurface;
pub(crate) mod layout;
pub(crate) mod scatter;
pub(crate) mod scatter_3d;
pub(crate) mod surface;
