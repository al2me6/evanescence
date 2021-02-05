use std::convert::TryFrom;
use std::fmt;
use std::iter;

use evanescence_core::geometry::Plane;
use evanescence_core::monte_carlo::Quality;
use evanescence_core::orbital::{self, Qn, RadialPlot};
use once_cell::sync::Lazy;
use strum::{Display, EnumIter};
use yew_state::SharedHandle;

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumIter, Display)]
pub(crate) enum Visualization {
    None,
    #[strum(serialize = "Radial wavefunction")]
    RadialWavefunction,
    #[strum(serialize = "Radial probability density")]
    RadialProbabilityDensity,
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
            Visualization::RadialProbabilityDensity => Ok(RadialPlot::ProbabilityDensity),
            Visualization::RadialProbabilityDistribution => Ok(RadialPlot::ProbabilityDistribution),
            _ => Err(format!("{:?} is not a radial plot.", value)),
        }
    }
}

static QN_PRESETS: Lazy<Vec<Qn>> = Lazy::new(|| Qn::enumerate_up_to_n(3).collect());

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum QnPreset {
    Preset(Qn),
    Custom,
}

impl Default for QnPreset {
    fn default() -> Self {
        Self::Preset(QN_PRESETS[0])
    }
}

impl QnPreset {
    pub(crate) fn iter() -> impl Iterator<Item = Self> {
        QN_PRESETS
            .iter()
            .map(|&qn| Self::Preset(qn))
            .chain(iter::once(Self::Custom))
    }
}

impl fmt::Display for QnPreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Preset(qn) => {
                let subscript =
                    orbital::wavefunctions::RealSphericalHarmonic::linear_combination_expression(
                        (*qn).into(),
                    )
                    .unwrap();
                write!(
                    f,
                    "{principal}{shell} {subscript}",
                    principal = qn.n(),
                    shell = orbital::subshell_name(qn.l()).unwrap(),
                    subscript = subscript
                )
            }
            Self::Custom => write!(f, "Custom"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub(crate) struct State {
    pub(crate) qn_preset: QnPreset,
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
