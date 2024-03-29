use std::f32::consts::{FRAC_PI_2, PI};

use super::layout::{Anchor, Title};
use crate::utils::b16_colors;

pub type ColorScale<'a> = &'a [(&'a str, &'a str)];

def_plotly_ty! {
    ColorBar<'a>

    x: f32 = 1.02,
    x_anchor as "xanchor": Anchor = Anchor::Left,
    outline_color as "outlinecolor": &'a str = b16_colors::BASE[0x06],
    thickness: u32 = 20,
    exponent_format as "exponentformat": &'a str = "power",
    #optional tick_vals as "tickvals": &'a [f32],
    #optional tick_text as "ticktext": &'a [&'a str],
    #optional title: Title<'a>,
}

#[allow(dead_code)]
/// Color scales extracted from Plotly.py.
pub mod color_scales {
    use super::ColorScale;

    pub const RD_BU_R: ColorScale = &[
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
        ("1.0", "rgb(103,0,31)"),
    ];

    pub const RD_YL_BU_R: ColorScale = &[
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
        ("1.0", "rgb(165,0,38)"),
    ];

    pub const GREENS: ColorScale = &[
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

    pub const PURP: ColorScale = &[
        ("0.0", "rgb(243, 224, 247)"),
        ("0.16666666666666666", "rgb(228, 199, 241)"),
        ("0.3333333333333333", "rgb(209, 175, 232)"),
        ("0.5", "rgb(185, 152, 221)"),
        ("0.6666666666666666", "rgb(159, 130, 206)"),
        ("0.8333333333333333", "rgb(130, 109, 186)"),
        ("1.0", "rgb(99, 88, 159)"),
    ];

    pub const ORANGES: ColorScale = &[
        ("0.0", "rgb(255,245,235)"),
        ("0.125", "rgb(254,230,206)"),
        ("0.25", "rgb(253,208,162)"),
        ("0.375", "rgb(253,174,107)"),
        ("0.5", "rgb(253,141,60)"),
        ("0.625", "rgb(241,105,19)"),
        ("0.75", "rgb(217,72,1)"),
        ("0.875", "rgb(166,54,3)"),
        ("1.0", "rgb(127,39,4)"),
    ];

    pub const PHASE: ColorScale = &[
        ("0.0", "rgb(167, 119, 12)"),
        ("0.09090909090909091", "rgb(197, 96, 51)"),
        ("0.18181818181818182", "rgb(217, 67, 96)"),
        ("0.2727272727272727", "rgb(221, 38, 163)"),
        ("0.36363636363636365", "rgb(196, 59, 224)"),
        ("0.4545454545454546", "rgb(153, 97, 244)"),
        ("0.5454545454545454", "rgb(95, 127, 228)"),
        ("0.6363636363636364", "rgb(40, 144, 183)"),
        ("0.7272727272727273", "rgb(15, 151, 136)"),
        ("0.8181818181818182", "rgb(39, 153, 79)"),
        ("0.9090909090909092", "rgb(119, 141, 17)"),
        ("1.0", "rgb(167, 119, 12)"),
    ];

    pub const TEMPO: ColorScale = &[
        ("0.0", "rgb(254, 245, 244)"),
        ("0.09090909090909091", "rgb(222, 224, 210)"),
        ("0.18181818181818182", "rgb(189, 206, 181)"),
        ("0.2727272727272727", "rgb(153, 189, 156)"),
        ("0.36363636363636365", "rgb(110, 173, 138)"),
        ("0.4545454545454546", "rgb(65, 157, 129)"),
        ("0.5454545454545454", "rgb(25, 137, 125)"),
        ("0.6363636363636364", "rgb(18, 116, 117)"),
        ("0.7272727272727273", "rgb(25, 94, 106)"),
        ("0.8181818181818182", "rgb(28, 72, 93)"),
        ("0.9090909090909092", "rgb(25, 51, 80)"),
        ("1.0", "rgb(20, 29, 67)"),
    ];
}

pub const PHASE_BAR_LABELS: &[&str] = &["−π", "−π/2", "0", "π/2", "π"];
pub const PHASE_BAR_TICKS: &[f32] = &[-PI, -FRAC_PI_2, 0.0, FRAC_PI_2, PI];
