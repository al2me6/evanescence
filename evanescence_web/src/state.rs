use evanescence_core::monte_carlo::Quality;
use evanescence_core::orbital::Qn;
use yew_state::SharedHandle;

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub(crate) struct State {
    pub(crate) qn: Qn,
    pub(crate) quality: Quality,
    pub(crate) nodes_show_radial: bool,
    pub(crate) nodes_show_angular: bool,
}

pub(crate) struct StateDiff {
    pub(crate) qn_or_quality: bool,
    pub(crate) nodes_radial: bool,
    pub(crate) nodes_angular: bool,
}

impl State {
    pub(crate) fn diff(&self, other: &Self) -> StateDiff {
        StateDiff {
            qn_or_quality: !(self.qn == other.qn && self.quality == other.quality),
            nodes_radial: self.nodes_show_radial != other.nodes_show_radial,
            nodes_angular: self.nodes_show_angular != other.nodes_show_angular,
        }
    }
}

pub(crate) type StateHandle = SharedHandle<State>;