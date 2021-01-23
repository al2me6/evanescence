use derivative::Derivative;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ModeBarButtons {
    AutoScale2d,
    HoverClosest3d,
    HoverCompareCartesian,
    Lasso2d,
    Pan2d,
    ResetCameraLastSave3d,
    ResetScale2d,
    Select2d,
    ToggleSpikelines,
    Zoom2d,
    ZoomIn2d,
    ZoomOut2d,
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
