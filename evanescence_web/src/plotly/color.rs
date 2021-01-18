use derivative::Derivative;
use serde::Serialize;

use crate::plotly::layout::Anchor;

pub(crate) type ColorScale<'a> = &'a [(&'a str, &'a str)];

#[derive(Serialize, Derivative)]
#[derivative(Default)]
pub(crate) struct ColorBar {
    #[derivative(Default(value = "1.02"))]
    pub(crate) x: f32,
    #[serde(rename = "xanchor")]
    #[derivative(Default(value = "Anchor::Left"))]
    pub(crate) x_anchor: Anchor,
}

#[allow(dead_code)]
pub(crate) mod color_scales {
    use super::ColorScale;

    pub(crate) const RED_BLUE_REVERSED: ColorScale = &[
        ("0.0", "rgb(5,48,97)"),
        ("0.1", "rgb(33,102,172)"),
        ("0.2", "rgb(67,147,195)"),
        ("0.3", "rgb(146,197,222)"),
        ("0.4", "rgb(209,229,240)"),
        ("0.5", "rgb(247,247,247)"),
        ("0.6", "rgb(253,219,199)"),
        ("0.7", "rgb(244,165,130)"),
        ("0.8", "rgb(214,96,77)"),
        ("0.9", "rgb(178,24,43)"),
        ("1.0", "rgb(103,0,31"),
    ];
    pub(crate) const RED_YELLOW_BLUE_REVERSED: ColorScale = &[
        ("0.0", "rgb(49,54,149)"),
        ("0.1", "rgb(69,117,180)"),
        ("0.2", "rgb(116,173,209)"),
        ("0.3", "rgb(171,217,233)"),
        ("0.4", "rgb(224,243,248)"),
        ("0.5", "rgb(255,255,191)"),
        ("0.6", "rgb(254,224,144)"),
        ("0.7", "rgb(253,174,97)"),
        ("0.8", "rgb(244,109,67)"),
        ("0.9", "rgb(215,48,39)"),
        ("1.0", "rgb(165,0,38"),
    ];

    pub(crate) const GREENS: ColorScale = &[
        ("0.0", "rgb(247,252,245)"),
        ("0.125", "rgb(229,245,224)"),
        ("0.25", "rgb(199,233,192)"),
        ("0.375", "rgb(161,217,155)"),
        ("0.5", "rgb(116,196,118)"),
        ("0.625", "rgb(65,171,93)"),
        ("0.75", "rgb(35,139,69)"),
        ("0.875", "rgb(0,109,44)"),
        ("1.0", "rgb(0,68,27)"),
    ];

    pub(crate) const PURP: ColorScale = &[
        ("0.0", "rgb(243, 224, 247)"),
        ("0.16666666666666666", "rgb(228, 199, 241)"),
        ("0.3333333333333333", "rgb(209, 175, 232)"),
        ("0.5", "rgb(185, 152, 221)"),
        ("0.6666666666666666", "rgb(159, 130, 206)"),
        ("0.8333333333333333", "rgb(130, 109, 186)"),
        ("1.0", "rgb(99, 88, 159)"),
    ];
}
