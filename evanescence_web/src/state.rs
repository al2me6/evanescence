use std::convert::{TryFrom, TryInto};
use std::fmt;

use evanescence_core::geometry::Plane;
use evanescence_core::monte_carlo::Quality;
use evanescence_core::orbital::wavefunctions::RealSphericalHarmonic;
use evanescence_core::orbital::{self, Qn, RadialPlot};
use getset::CopyGetters;
use once_cell::sync::Lazy;
use strum::{Display, EnumDiscriminants, EnumIter, IntoEnumIterator};
use yew_state::SharedHandle;

#[allow(clippy::upper_case_acronyms)] // "XY", etc. are not acronyms.
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
pub(crate) struct QnPreset(usize);

impl Default for QnPreset {
    fn default() -> Self {
        Self(0)
    }
}

impl QnPreset {
    pub(crate) fn iter() -> impl Iterator<Item = Self> {
        (0..QN_PRESETS.len()).map(Self)
    }
}

impl From<QnPreset> for Qn {
    fn from(preset: QnPreset) -> Self {
        QN_PRESETS[preset.0]
    }
}

impl TryFrom<Qn> for QnPreset {
    type Error = ();
    fn try_from(qn: Qn) -> Result<Self, Self::Error> {
        for (idx, preset) in QN_PRESETS.iter().enumerate() {
            if preset == &qn {
                return Ok(Self(idx));
            }
        }
        Err(())
    }
}

impl fmt::Display for QnPreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let qn: Qn = (*self).into();
        let subscript = RealSphericalHarmonic::expression(qn.into()).unwrap();
        write!(
            f,
            "{principal}{shell} {subscript}",
            principal = qn.n(),
            shell = orbital::subshell_name(qn.l()).unwrap(),
            subscript = subscript
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct RealSimpleState {
    preset: QnPreset,
    nodes_rad: bool,
    nodes_ang: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct RealState {
    qn: Qn,
    nodes_rad: bool,
    nodes_ang: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ComplexState {
    qn: Qn,
}

#[derive(Clone, PartialEq, Eq, Debug, EnumDiscriminants)]
#[strum_discriminants(vis(pub(crate)), name(Mode), derive(EnumIter))]
enum StateInner {
    RealSimple(RealSimpleState),
    Real(RealState),
    Complex(ComplexState),
}

use StateInner::*;

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RealSimple => write!(f, "Real (Simple)"),
            Self::Real => write!(f, "Real (Full)"),
            Self::Complex => write!(f, "Complex"),
        }
    }
}

impl Default for StateInner {
    fn default() -> Self {
        RealSimple(RealSimpleState::default())
    }
}

impl StateInner {
    fn transition(&mut self, mode: Mode) {
        match (&self, mode) {
            (RealSimple(state), Mode::Real) => {
                *self = Real(RealState {
                    qn: state.preset.into(),
                    nodes_rad: state.nodes_rad,
                    nodes_ang: state.nodes_ang,
                });
            }
            (Real(state), Mode::RealSimple) => {
                log::info!("Transition from Real (Full) to Real (Simple) is possibly lossy.");
                *self = RealSimple(RealSimpleState {
                    preset: state.qn.try_into().unwrap_or_default(),
                    nodes_rad: state.nodes_rad,
                    nodes_ang: state.nodes_ang,
                });
            }
            (RealSimple(state), Mode::Complex) => {
                *self = Complex(ComplexState {
                    qn: state.preset.into(),
                });
            }
            (Real(state), Mode::Complex) => {
                *self = Complex(ComplexState { qn: state.qn });
            }
            (Complex(state), Mode::Real) => {
                *self = Real(RealState {
                    qn: state.qn,
                    nodes_rad: false,
                    nodes_ang: false,
                });
            }
            (Complex(state), Mode::RealSimple) => {
                log::info!("Transition from Complex to Real (Simple) is possibly lossy.");
                *self = RealSimple(RealSimpleState {
                    preset: state.qn.try_into().unwrap_or_default(),
                    nodes_rad: false,
                    nodes_ang: false,
                });
            }
            // Same state, do nothing.
            (state, mode) => {
                assert!(Mode::from(state as &_) == mode);
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, CopyGetters, Default)]
pub(crate) struct State {
    state: StateInner,
    #[getset(get_copy = "pub(crate)")]
    quality: Quality,
    supplement: Visualization,
}

/// Queries.
impl State {
    pub(crate) fn mode(&self) -> Mode {
        (&self.state).into()
    }

    pub(crate) fn is_real(&self) -> bool {
        matches!(self.state, RealSimple(_) | Real(_))
    }

    pub(crate) fn available_supplements(&self) -> Vec<Visualization> {
        match &self.state {
            RealSimple(_) | Real(_) => Visualization::iter().collect(),
            Complex(_) => vec![
                Visualization::None,
                Visualization::RadialWavefunction,
                Visualization::RadialProbabilityDensity,
                Visualization::RadialProbabilityDistribution,
            ],
        }
    }

    pub(crate) fn supplement_is_cross_section(&self) -> bool {
        [
            Visualization::CrossSectionXY,
            Visualization::CrossSectionYZ,
            Visualization::CrossSectionZX,
        ]
        .contains(&self.supplement)
    }

    pub(crate) fn is_new_orbital(&self, other: &Self) -> bool {
        match (self.mode(), other.mode()) {
            (m1, m2) if m1 == m2 => self.qn() != other.qn(),
            (Mode::RealSimple, Mode::Real) | (Mode::Real, Mode::RealSimple) => {
                self.qn() != other.qn()
            }
            _ => true,
        }
    }
}

/// Getters for specific states.
impl State {
    pub(crate) fn supplement(&self) -> Visualization {
        assert!(self.available_supplements().contains(&self.supplement));
        self.supplement
    }

    pub(crate) fn qn(&self) -> Qn {
        match &self.state {
            RealSimple(state) => state.preset.into(),
            Real(state) => state.qn,
            Complex(state) => state.qn,
        }
    }

    pub(crate) fn qn_mut(&mut self) -> &mut Qn {
        match &mut self.state {
            RealSimple(_) => unreachable!(),
            Real(state) => &mut state.qn,
            Complex(state) => &mut state.qn,
        }
    }

    pub(crate) fn preset(&self) -> QnPreset {
        match &self.state {
            RealSimple(state) => state.preset,
            _ => unreachable!(),
        }
    }

    pub(crate) fn nodes_rad(&self) -> bool {
        match &self.state {
            RealSimple(state) => state.nodes_rad,
            Real(state) => state.nodes_rad,
            Complex(_) => false,
        }
    }

    pub(crate) fn nodes_ang(&self) -> bool {
        match &self.state {
            RealSimple(state) => state.nodes_ang,
            Real(state) => state.nodes_ang,
            Complex(_) => false,
        }
    }
}

/// Setters for specific states.
impl State {
    pub(crate) fn set_mode(&mut self, mode: Mode) {
        self.state.transition(mode);
        if !self.available_supplements().contains(&self.supplement) {
            self.supplement = Visualization::None;
        }
    }

    pub(crate) fn set_quality(&mut self, quality: Quality) {
        self.quality = quality;
    }

    pub(crate) fn set_supplement(&mut self, supplement: Visualization) {
        assert!(self.available_supplements().contains(&supplement));
        self.supplement = supplement;
    }

    pub(crate) fn set_preset(&mut self, preset: QnPreset) {
        match &mut self.state {
            RealSimple(state) => {
                state.preset = preset;
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn set_nodes_rad(&mut self, visibility: bool) {
        match &mut self.state {
            RealSimple(state) => state.nodes_rad = visibility,
            Real(state) => state.nodes_rad = visibility,
            Complex(_) => unreachable!(),
        }
    }

    pub(crate) fn set_nodes_ang(&mut self, visibility: bool) {
        match &mut self.state {
            RealSimple(state) => state.nodes_ang = visibility,
            Real(state) => state.nodes_ang = visibility,
            Complex(_) => unreachable!(),
        }
    }
}

pub(crate) type StateHandle = SharedHandle<State>;
