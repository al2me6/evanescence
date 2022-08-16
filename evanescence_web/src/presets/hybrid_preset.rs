use std::f32::consts::{FRAC_1_SQRT_2, SQRT_2};
use std::fmt;
use std::sync::LazyLock;

use evanescence_core::numerics::consts::{FRAC_1_SQRT_3, FRAC_1_SQRT_6, SQRT_3};
use evanescence_core::orbital::hybrid::Kind;
use evanescence_core::{kind, lc};

use super::{Preset, PresetLibrary};

#[allow(clippy::too_many_lines)] // Data.
static HYBRID_PRESETS: LazyLock<Vec<Kind>> = LazyLock::new(|| {
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

impl PresetLibrary for Preset<Kind> {
    type Item = Kind;

    fn library() -> &'static [Self::Item] {
        &HYBRID_PRESETS
    }
}

impl fmt::Display for Preset<Kind> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.item())
    }
}
