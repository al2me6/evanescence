use derivative::Derivative;
use serde::Serialize;

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
