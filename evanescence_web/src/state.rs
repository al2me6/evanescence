use evanescence_core::monte_carlo::Quality;
use evanescence_core::orbital::Qn;
use strum::{Display, EnumIter};
use yew_state::SharedHandle;

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumIter, Display)]
pub(crate) enum Visualization {
    None,
    #[strum(serialize = "Radial wavefunction")]
    RadialWavefunction,
    #[strum(serialize = "Radial probability")]
    RadialProbability,
    #[strum(serialize = "Radial probability distribution")]
    RadialProbabilityDistribution,
    #[strum(serialize = "XY-plane cross section")]
    CrossSectionXY,
    #[strum(serialize = "YZ-plane cross section")]
    CrossSectionYZ,
    #[strum(serialize = "ZX-plane cross-section")]
    CrossSectionZX,
}

impl Default for Visualization {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub(crate) struct State {
    pub(crate) qn: Qn,
    pub(crate) quality: Quality,
    pub(crate) nodes_show_radial: bool,
    pub(crate) nodes_show_angular: bool,
    pub(crate) extra_visualization: Visualization,
}

pub(crate) struct StateDiff {
    pub(crate) qn_or_quality: bool,
    pub(crate) nodes_radial: bool,
    pub(crate) nodes_angular: bool,
    pub(crate) extra_visualization: bool,
}

impl State {
    pub(crate) fn diff(&self, other: &Self) -> StateDiff {
        StateDiff {
            qn_or_quality: !(self.qn == other.qn && self.quality == other.quality),
            nodes_radial: self.nodes_show_radial != other.nodes_show_radial,
            nodes_angular: self.nodes_show_angular != other.nodes_show_angular,
            extra_visualization: self.extra_visualization != other.extra_visualization,
        }
    }
}

pub(crate) type StateHandle = SharedHandle<State>;
