use std::convert::TryFrom;
use std::f32::consts::{FRAC_1_SQRT_2, SQRT_2};
use std::fmt;

use evanescence_core::geometry::Vec3;
use evanescence_core::orbital::atomic::RealSphericalHarmonic;
use evanescence_core::orbital::hybrid::{Component, Kind};
use evanescence_core::orbital::molecular::{Element, Geometry, Lcao, LcaoAtom};
use evanescence_core::orbital::{self, Qn};
use evanescence_core::{kind, lc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const FRAC_1_SQRT_3: f32 = 0.577_350_3;
const FRAC_1_SQRT_6: f32 = 0.408_248_3;
const SQRT_3: f32 = 1.732_050_8;

static QN_PRESETS: Lazy<Vec<Qn>> = Lazy::new(|| Qn::enumerate_up_to_n(3).unwrap().collect());

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct QnPreset(usize);

impl Default for QnPreset {
    fn default() -> Self {
        Self(0)
    }
}

impl QnPreset {
    pub fn iter() -> impl Iterator<Item = Self> {
        (0..QN_PRESETS.len()).map(Self)
    }

    /// Try to convert an arbitrary [`Qn`] to a preset that has similar characteristics, falling
    /// back to 1s if that fails.
    pub fn from_qn_lossy(mut qn: Qn) -> Self {
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
        Err(format!("({}) is not a valid preset", qn))
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
            shell = orbital::atomic::subshell_name(qn.l()).expect("failed to get subshell name"),
            subscript = subscript,
        )
    }
}

#[allow(clippy::too_many_lines)] // Data.
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
        kind! {
            mixture: {
                n: 3,
                0 => 1,
                1 => 3,
                2 => 1,
            },
            symmetry: "trigonal bipyramidal",
            description: "axial",
            combinations: {
                lc! {
                    overall: FRAC_1_SQRT_2,
                    (3, 1, 0) * 1.0,
                    (3, 2, 0) * 1.0,
                },
                lc! {
                    overall: FRAC_1_SQRT_2,
                    (3, 1, 0) * 1.0,
                    (3, 2, 0) * -1.0,
                },
                lc! {
                    overall: FRAC_1_SQRT_3,
                    (3, 0, 0) * 1.0,
                    (3, 1, 1) * SQRT_2,
                },
                lc! {
                    overall: FRAC_1_SQRT_3,
                    (3, 0, 0) * 1.0,
                    (3, 1, 1) * -FRAC_1_SQRT_2,
                    (3, 1, -1) * SQRT_3 * FRAC_1_SQRT_2,
                },
                lc! {
                    overall: FRAC_1_SQRT_3,
                    (3, 0, 0) * 1.0,
                    (3, 1, 1) * -FRAC_1_SQRT_2,
                    (3, 1, -1) * -SQRT_3 * FRAC_1_SQRT_2,
                },
            },
        },
        kind! {
            mixture: {
                n: 3,
                0 => 1,
                1 => 3,
                2 => 1,
            },
            symmetry: "trigonal bipyramidal",
            description: "equatorial",
            combinations: {
                lc! {
                    overall: FRAC_1_SQRT_3,
                    (3, 0, 0) * 1.0,
                    (3, 1, 1) * SQRT_2,
                },
                lc! {
                    overall: FRAC_1_SQRT_3,
                    (3, 0, 0) * 1.0,
                    (3, 1, 1) * -FRAC_1_SQRT_2,
                    (3, 1, -1) * SQRT_3 * FRAC_1_SQRT_2,
                },
                lc! {
                    overall: FRAC_1_SQRT_3,
                    (3, 0, 0) * 1.0,
                    (3, 1, 1) * -FRAC_1_SQRT_2,
                    (3, 1, -1) * -SQRT_3 * FRAC_1_SQRT_2,
                },
                lc! {
                    overall: FRAC_1_SQRT_2,
                    (3, 1, 0) * 1.0,
                    (3, 2, 0) * 1.0,
                },
                lc! {
                    overall: FRAC_1_SQRT_2,
                    (3, 1, 0) * 1.0,
                    (3, 2, 0) * -1.0,
                },
            },
        },
        kind! {
            mixture: {
                n: 3,
                0 => 1,
                1 => 3,
                2 => 2,
            },
            symmetry: "octahedral",
            combinations: {
                lc! {
                    overall: FRAC_1_SQRT_6,
                    (3, 0, 0) * 1.0,
                    (3, 1, 0) * SQRT_3,
                    (3, 2, 0) * SQRT_2,
                },
                lc! {
                    overall: FRAC_1_SQRT_6,
                    (3, 0, 0) * 1.0,
                    (3, 1, 1) * SQRT_3,
                    (3, 2, 0) * -FRAC_1_SQRT_2,
                    (3, 2, 2) * SQRT_3 * FRAC_1_SQRT_2,
                },
                lc! {
                    overall: FRAC_1_SQRT_6,
                    (3, 0, 0) * 1.0,
                    (3, 1, -1) * SQRT_3,
                    (3, 2, 0) * -FRAC_1_SQRT_2,
                    (3, 2, 2) * -SQRT_3 * FRAC_1_SQRT_2,
                },
                lc! {
                    overall: FRAC_1_SQRT_6,
                    (3, 0, 0) * 1.0,
                    (3, 1, 1) * -SQRT_3,
                    (3, 2, 0) * -FRAC_1_SQRT_2,
                    (3, 2, 2) * SQRT_3 * FRAC_1_SQRT_2,
                },
                lc! {
                    overall: FRAC_1_SQRT_6,
                    (3, 0, 0) * 1.0,
                    (3, 1, -1) * -SQRT_3,
                    (3, 2, 0) * -FRAC_1_SQRT_2,
                    (3, 2, 2) * -SQRT_3 * FRAC_1_SQRT_2,
                },
                lc! {
                    overall: FRAC_1_SQRT_6,
                    (3, 0, 0) * 1.0,
                    (3, 1, 0) * -SQRT_3,
                    (3, 2, 0) * SQRT_2,
                },
            },
        },
    ]
});

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct HybridPreset(usize);

impl HybridPreset {
    pub fn iter() -> impl Iterator<Item = Self> {
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

pub struct ProtoDiatomicLcao(Lcao);

impl ProtoDiatomicLcao {
    pub fn with_separation(&self, sep: f32) -> Lcao {
        let mut lcao = self.0.clone();
        lcao.combination[0].offset = Vec3::new(0.0, 0.0, -sep / 2.0);
        lcao.combination[1].offset = Vec3::new(0.0, 0.0, sep / 2.0);
        lcao
    }
}

macro_rules! lcao_diatomic {
    (
        $(
            $(* $geometry_star:ident)? $($geometry:ident)? {
                $elem1:ident @ ($(($n1:literal $l1:literal $m1:literal) * $w1: expr);+),
                $elem2:ident @ ($(($n2:literal $l2:literal $m2:literal) * $w2: expr);+)
                $(,)?
            }
        ),+
        $(,)?
    ) => {
        vec![
            $(
                ProtoDiatomicLcao(
                    Lcao {
                        bonding: lcao_diatomic!(@bonding $($geometry_star)?),
                        geometry: Geometry:: $($geometry_star)? $($geometry)?,
                        combination: vec![
                            lcao_diatomic!(@atom $elem1 $($n1 $l1 $m1 $w1);+),
                            lcao_diatomic!(@atom $elem2 $($n2 $l2 $m2 $w2);+),
                        ]
                    }
                )
            ),+
        ]
    };
    (@atom $elem:ident $($n:literal $l:literal $m:literal $w:expr);+) => {
        LcaoAtom {
            elem: Element::$elem,
            offset: Vec3::ZERO,
            orbitals: vec![
                $(Component {
                    qn: Qn::new($n, $l, $m).unwrap(),
                    weight: $w,
                }),+
            ],
        }
    };
    (@bonding) => { true };
    (@bonding $geom:ident) => { false };
}

static MO_PRESETS: Lazy<Vec<ProtoDiatomicLcao>> = Lazy::new(|| {
    lcao_diatomic! {
        Sigma {
            H @ ((1 0 0) * FRAC_1_SQRT_2),
            H @ ((1 0 0) * FRAC_1_SQRT_2),
        },
        * Sigma {
            H @ ((1 0 0) * FRAC_1_SQRT_2),
            H @ ((1 0 0) * -FRAC_1_SQRT_2),
        },
        Sigma {
            H @ ((2 0 0) * FRAC_1_SQRT_2),
            H @ ((2 0 0) * FRAC_1_SQRT_2),
        },
        * Sigma {
            H @ ((2 0 0) * FRAC_1_SQRT_2),
            H @ ((2 0 0) * -FRAC_1_SQRT_2),
        },
        Sigma {
            H @ ((2 1 0) * FRAC_1_SQRT_2),
            H @ ((2 1 0) * -FRAC_1_SQRT_2),
        },
        Pi {
            H @ ((2 1 1) * FRAC_1_SQRT_2),
            H @ ((2 1 1) * FRAC_1_SQRT_2),
        },
        Pi {
            H @ ((2 1 -1) * FRAC_1_SQRT_2),
            H @ ((2 1 -1) * FRAC_1_SQRT_2),
        },
        * Pi {
            H @ ((2 1 1) * FRAC_1_SQRT_2),
            H @ ((2 1 1) * -FRAC_1_SQRT_2),
        },
        * Pi {
            H @ ((2 1 -1) * FRAC_1_SQRT_2),
            H @ ((2 1 -1) * -FRAC_1_SQRT_2),
        },
        * Sigma {
            H @ ((2 1 0) * FRAC_1_SQRT_2),
            H @ ((2 1 0) * FRAC_1_SQRT_2),
        },
        // Sigma {
        //     C @ ((2 0 0) * -0.1389 ; (2 1 0) * 0.0080),
        //     O @ ((2 0 0) *  0.6492 ; (2 1 0) * 0.7478),
        // },
    }
});

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct MoPreset(usize);

impl MoPreset {
    pub fn iter() -> impl Iterator<Item = Self> {
        (0..MO_PRESETS.len()).map(Self)
    }
}

impl Default for MoPreset {
    fn default() -> Self {
        Self(0)
    }
}

impl From<MoPreset> for &'static ProtoDiatomicLcao {
    fn from(preset: MoPreset) -> Self {
        &MO_PRESETS[preset.0]
    }
}

impl fmt::Display for MoPreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &MO_PRESETS[self.0].0)
    }
}
