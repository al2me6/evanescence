use std::fmt;
use std::sync::LazyLock;

use evanescence_core::numerics::special::spherical_harmonics::RealSphericalHarmonic;
use evanescence_core::orbital::{self, Qn};
use evanescence_core::utils::sup_sub_string::SupSubFormat;

use super::{Preset, PresetLibrary};

static QN_PRESETS: LazyLock<Vec<Qn>> =
    LazyLock::new(|| Qn::enumerate_up_to_n(3).unwrap().collect());

impl PresetLibrary for Preset<Qn> {
    type Item = Qn;

    fn library() -> &'static [Self::Item] {
        &QN_PRESETS
    }
}

impl Preset<Qn> {
    /// Try to convert an arbitrary [`Qn`] to a preset that has similar characteristics, falling
    /// back to 1s if that fails.
    pub fn find_closest_match(mut qn: Qn) -> Self {
        Self::try_find(&qn).unwrap_or_else(|| {
            qn.set_n_clamping(Self::library().last().unwrap().n())
                .unwrap();
            Self::try_find(&qn).unwrap_or_default()
        })
    }
}

impl fmt::Display for Preset<Qn> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let qn = self.item();
        let subscript =
            RealSphericalHarmonic::expression(qn.into()).expect("failed to get expression");
        write!(
            f,
            "{principal}{shell} {subscript}",
            principal = qn.n(),
            shell = orbital::atomic::subshell_name(qn.l()).expect("failed to get subshell name"),
            subscript = subscript.format(SupSubFormat::Unicode).unwrap(),
        )
    }
}
