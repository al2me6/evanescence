use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
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

def_plotly_ty! {
    #[serde(rename_all = "camelCase")]
    Config<'a>

    display_logo as "displaylogo": bool,
    mode_bar_buttons_to_remove: &'a [ModeBarButtons],
    responsive: bool = true,
}
