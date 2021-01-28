use std::convert::TryFrom;

use evanescence_core::geometry::Plane;
use evanescence_core::monte_carlo::Quality;
use evanescence_core::orbital::{Qn, RadialPlot};
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
    #[strum(serialize = "3D isosurface")]
    Isosurface3D,
}

impl Default for Visualization {
    fn default() -> Self {
        Self::None
    }
}

impl TryFrom<Visualization> for Plane {
    type Error = String;

    fn try_from(value: Visualization) -> Result<Self, Self::Error> {
        match value {
            Visualization::CrossSectionXY => Ok(Plane::XY),
            Visualization::CrossSectionYZ => Ok(Plane::YZ),
            Visualization::CrossSectionZX => Ok(Plane::ZX),
            _ => Err(format!("{:?} does not have an associated plane.", value)),
        }
    }
}

impl TryFrom<Visualization> for RadialPlot {
    type Error = String;

    fn try_from(value: Visualization) -> Result<Self, Self::Error> {
        match value {
            Visualization::RadialWavefunction => Ok(RadialPlot::Wavefunction),
            Visualization::RadialProbability => Ok(RadialPlot::Probability),
            Visualization::RadialProbabilityDistribution => Ok(RadialPlot::ProbabilityDistribution),
            _ => Err(format!("{:?} is not a radial plot.", value)),
        }
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
    pub(crate) cross_section: bool,
}

impl State {
    pub(crate) fn cross_section_enabled(&self) -> bool {
        [
            Visualization::CrossSectionXY,
            Visualization::CrossSectionYZ,
            Visualization::CrossSectionZX,
        ]
        .contains(&self.extra_visualization)
    }

    pub(crate) fn diff(&self, other: &Self) -> StateDiff {
        let extra_visualization = self.extra_visualization != other.extra_visualization;
        StateDiff {
            qn_or_quality: !(self.qn == other.qn && self.quality == other.quality),
            nodes_radial: self.nodes_show_radial != other.nodes_show_radial,
            nodes_angular: self.nodes_show_angular != other.nodes_show_angular,
            extra_visualization,
            cross_section: extra_visualization
                && (self.cross_section_enabled() || other.cross_section_enabled()),
        }
    }
}

pub(crate) type StateHandle = SharedHandle<State>;
