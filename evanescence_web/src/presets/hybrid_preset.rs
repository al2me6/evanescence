use std::fmt;
use std::sync::LazyLock;

use evanescence_core::orbital::hybrid::{library, Kind};

use super::{Preset, PresetLibrary};

static HYBRID_PRESETS: LazyLock<Vec<Kind>> = LazyLock::new(|| {
    use library::SP3D;

    // Note that two copies of sp3d orbitals are made here to allow both geometries to be
    // displayed, as the archetype is always the combination sampled.
    let mut sp3d_axial = SP3D.clone();
    sp3d_axial.set_description(Some("axial".to_owned()));

    let mut sp3d_equatorial_combinations = SP3D.combinations().clone();
    sp3d_equatorial_combinations.rotate_left(2); // First two are axial.
    let sp3d_equatorial = Kind::new(
        SP3D.n(),
        SP3D.mixture().clone(),
        SP3D.symmetry().clone(),
        Some("equatorial".to_string()),
        sp3d_equatorial_combinations,
    )
    .unwrap();

    vec![
        library::SP.clone(),
        library::SP2.clone(),
        library::SP3.clone(),
        sp3d_axial,
        sp3d_equatorial,
        library::SP3D2.clone(),
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
