use core::fmt;
use std::f32::consts::FRAC_1_SQRT_2;

use evanescence_core::geometry::Vec3;
use evanescence_core::orbital::hybrid::Component;
use evanescence_core::orbital::molecular::{Element, Geometry, Lcao, LcaoAtom};
use evanescence_core::orbital::Qn;
use once_cell::sync::Lazy;

use super::{Preset, PresetLibrary};

#[derive(PartialEq)]
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

impl PresetLibrary for Preset<ProtoDiatomicLcao> {
    type Item = ProtoDiatomicLcao;

    fn library() -> &'static [Self::Item] {
        &MO_PRESETS
    }
}

impl fmt::Display for Preset<ProtoDiatomicLcao> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.item().0)
    }
}
