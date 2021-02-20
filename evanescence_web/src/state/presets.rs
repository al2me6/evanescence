use std::convert::TryFrom;
use std::f32::consts::{FRAC_1_SQRT_2, SQRT_2};
use std::fmt;

use evanescence_core::lc;
use evanescence_core::orbital::wavefunctions::RealSphericalHarmonic;
use evanescence_core::orbital::{self, LinearCombination, Qn};
use getset::Getters;
use once_cell::sync::Lazy;

const FRAC_1_SQRT_3: f32 = 0.5773503;
const FRAC_1_SQRT_6: f32 = 0.4082483;
const SQRT_3: f32 = 1.7320508;

static QN_PRESETS: Lazy<Vec<Qn>> = Lazy::new(|| Qn::enumerate_up_to_n(3).collect());

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct QnPreset(usize);

impl Default for QnPreset {
    fn default() -> Self {
        Self(0)
    }
}

impl QnPreset {
    pub(crate) fn iter() -> impl Iterator<Item = Self> {
        (0..QN_PRESETS.len()).map(Self)
    }
}

impl From<QnPreset> for Qn {
    fn from(preset: QnPreset) -> Self {
        QN_PRESETS[preset.0]
    }
}

impl From<QnPreset> for &'static Qn {
    fn from(preset: QnPreset) -> Self {
        &QN_PRESETS[preset.0]
    }
}

impl TryFrom<Qn> for QnPreset {
    type Error = String;

    fn try_from(qn: Qn) -> Result<Self, Self::Error> {
        for (idx, preset) in QN_PRESETS.iter().enumerate() {
            if preset == &qn {
                return Ok(Self(idx));
            }
        }
        Err(format!("{} is not a valid preset", qn))
    }
}

impl fmt::Display for QnPreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let qn: Qn = (*self).into();
        let subscript =
            RealSphericalHarmonic::expression(&qn.into()).expect("failed to get expression");
        write!(
            f,
            "{principal}{shell} {subscript}",
            principal = qn.n(),
            shell = orbital::subshell_name(qn.l()).expect("failed to get subshell name"),
            subscript = subscript
        )
    }
}

#[derive(Clone, PartialEq, Getters)]
#[getset(get = "pub(crate)")]
pub(crate) struct HybridKind {
    kind: String,
    symmetry: String,
    principal: LinearCombination,
    rotations: Vec<LinearCombination>,
}

impl HybridKind {
    pub(crate) fn count(&self) -> usize {
        self.rotations.len() + 1
    }
}

impl fmt::Display for HybridKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.kind)
    }
}

static HYBRID_PRESETS: Lazy<Vec<HybridKind>> = Lazy::new(|| {
    vec![
        HybridKind {
            kind: "sp".to_owned(),
            symmetry: "linear".to_owned(),
            principal: lc! {
                kind = "sp",
                overall = FRAC_1_SQRT_2,
                (2, 0, 0) => 1.0,
                (2, 1, 0) => 1.0,
            },
            rotations: vec![lc! {
                kind = "sp",
                overall = FRAC_1_SQRT_2,
                (2, 0, 0) => 1.0,
                (2, 1, 0) => -1.0,
            }],
        },
        HybridKind {
            kind: "sp²".to_owned(),
            symmetry: "trigonal planar".to_owned(),
            principal: lc! {
                kind = "sp²",
                overall = FRAC_1_SQRT_3,
                (2, 0, 0) => 1.0,
                (2, 1, 1) => -SQRT_2,
            },
            rotations: vec![
                lc! {
                    kind = "sp²",
                    overall = FRAC_1_SQRT_6,
                    (2, 0, 0) => SQRT_2,
                    (2, 1, 1) => 1.0,
                    (2, 1, -1) => SQRT_3,
                },
                lc! {
                    kind = "sp²",
                    overall = FRAC_1_SQRT_6,
                    (2, 0, 0) => SQRT_2,
                    (2, 1, 1) => 1.0,
                    (2, 1, -1) => -SQRT_3,
                },
            ],
        },
        HybridKind {
            kind: "sp³".to_owned(),
            symmetry: "tetrahedral".to_owned(),
            principal: lc! {
                kind = "sp³",
                overall = 0.5,
                (2, 0, 0) => 1.0,
                (2, 1, 1) => 1.0,
                (2, 1, -1) => 1.0,
                (2, 1, 0) => 1.0,
            },
            rotations: vec![
                lc! {
                    kind = "sp³",
                    overall = 0.5,
                    (2, 0, 0) => 1.0,
                    (2, 1, 1) => -1.0,
                    (2, 1, -1) => -1.0,
                    (2, 1, 0) => 1.0,
                },
                lc! {
                    kind = "sp³",
                    overall = 0.5,
                    (2, 0, 0) => 1.0,
                    (2, 1, 1) => 1.0,
                    (2, 1, -1) => -1.0,
                    (2, 1, 0) => -1.0,
                },
                lc! {
                    kind = "sp³",
                    overall = 0.5,
                    (2, 0, 0) => 1.0,
                    (2, 1, 1) => -1.0,
                    (2, 1, -1) => 1.0,
                    (2, 1, 0) => -1.0,
                },
            ],
        },
    ]
});

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct HybridPreset(usize);

impl HybridPreset {
    pub(crate) fn iter() -> impl Iterator<Item = Self> {
        (0..HYBRID_PRESETS.len()).map(Self)
    }
}

impl Default for HybridPreset {
    fn default() -> Self {
        Self(0)
    }
}

impl From<HybridPreset> for &'static HybridKind {
    fn from(preset: HybridPreset) -> Self {
        &HYBRID_PRESETS[preset.0]
    }
}

impl fmt::Display for HybridPreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", HYBRID_PRESETS[self.0])
    }
}
