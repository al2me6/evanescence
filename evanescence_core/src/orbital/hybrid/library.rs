//! Collection of common hybridizations.

use std::f32::consts::{FRAC_1_SQRT_2, SQRT_2};
use std::sync::LazyLock;

use super::Kind;
use crate::numerics::consts::{FRAC_1_SQRT_3, FRAC_1_SQRT_6, SQRT_3};

pub static SP: LazyLock<Kind> = LazyLock::new(|| {
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
    }
});

pub static SP2: LazyLock<Kind> = LazyLock::new(|| {
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
    }
});

pub static SP3: LazyLock<Kind> = LazyLock::new(|| {
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
    }
});

pub static SP3D: LazyLock<Kind> = LazyLock::new(|| {
    kind! {
        mixture: {
            n: 3,
            0 => 1,
            1 => 3,
            2 => 1,
        },
        symmetry: "trigonal bipyramidal",
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
    }
});

pub static SP3D2: LazyLock<Kind> = LazyLock::new(|| {
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
    }
});
