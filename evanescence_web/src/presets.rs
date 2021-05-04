use std::convert::TryFrom;
use std::f32::consts::{FRAC_1_SQRT_2, SQRT_2};
use std::fmt;

use evanescence_core::orbital::hybrid::Kind;
use evanescence_core::orbital::wavefunctions::RealSphericalHarmonic;
use evanescence_core::orbital::{self, Qn};
use evanescence_core::{kind, lc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const FRAC_1_SQRT_3: f32 = 0.577_350_3;
const FRAC_1_SQRT_6: f32 = 0.408_248_3;
const SQRT_3: f32 = 1.732_050_8;

static QN_PRESETS: Lazy<Vec<Qn>> = Lazy::new(|| Qn::enumerate_up_to_n(3).unwrap().collect());

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
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

    /// Try to convert an arbitrary [`Qn`] to a preset that has similar characteristics, falling
    /// back to 1s if that fails.
    pub(crate) fn from_qn_lossy(mut qn: Qn) -> Self {
        Self::try_from(qn).unwrap_or_else(|_| {
            qn.set_n_clamping(QN_PRESETS.last().unwrap().n()).unwrap();
            Self::try_from(qn).unwrap_or_default()
        })
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

static HYBRID_PRESETS: Lazy<Vec<Kind>> = Lazy::new(|| {
    vec![
        kind! {
            mixture: {
                n: 2,
                0 => 1,
                1 => 1,
            },
            symmetry: "linear",
            combinations: {
                lc! {
                    overall: FRAC_1_SQRT_2,
                    (2, 0, 0) * 1.0,
                    (2, 1, 0) * 1.0,
                },
                lc! {
                    overall: FRAC_1_SQRT_2,
                    (2, 0, 0) * 1.0,
                    (2, 1, 0) * -1.0,
                },
            }
        },
        kind! {
            mixture: {
                n: 2,
                0 => 1,
                1 => 2,
            },
            symmetry: "trigonal planar",
            combinations: {
                lc! {
                    overall: FRAC_1_SQRT_3,
                    (2, 0, 0) * 1.0,
                    (2, 1, 1) * -SQRT_2,
                },
                lc! {
                    overall: FRAC_1_SQRT_6,
                    (2, 0, 0) * SQRT_2,
                    (2, 1, 1) * 1.0,
                    (2, 1, -1) * SQRT_3,
                },
                lc! {
                    overall: FRAC_1_SQRT_6,
                    (2, 0, 0) * SQRT_2,
                    (2, 1, 1) * 1.0,
                    (2, 1, -1) * -SQRT_3,
                },
            }
        },
        kind! {
            mixture: {
                n: 2,
                0 => 1,
                1 => 3,
            },
            symmetry: "tetrahedral",
            combinations: {
                lc! {
                    overall: 0.5,
                    (2, 0, 0) * 1.0,
                    (2, 1, 1) * 1.0,
                    (2, 1, -1) * 1.0,
                    (2, 1, 0) * 1.0,
                },
                lc! {
                    overall: 0.5,
                    (2, 0, 0) * 1.0,
                    (2, 1, 1) * -1.0,
                    (2, 1, -1) * -1.0,
                    (2, 1, 0) * 1.0,
                },
                lc! {
                    overall: 0.5,
                    (2, 0, 0) * 1.0,
                    (2, 1, 1) * 1.0,
                    (2, 1, -1) * -1.0,
                    (2, 1, 0) * -1.0,
                },
                lc! {
                    overall: 0.5,
                    (2, 0, 0) * 1.0,
                    (2, 1, 1) * -1.0,
                    (2, 1, -1) * 1.0,
                    (2, 1, 0) * -1.0,
                },
            },
        },
    ]
});

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
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

impl From<HybridPreset> for &'static Kind {
    fn from(preset: HybridPreset) -> Self {
        &HYBRID_PRESETS[preset.0]
    }
}

impl fmt::Display for HybridPreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", HYBRID_PRESETS[self.0])
    }
}
