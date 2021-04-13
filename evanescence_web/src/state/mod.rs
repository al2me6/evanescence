mod presets;

use std::convert::TryFrom;
use std::fmt;

use evanescence_core::geometry::{ComponentForm, GridValues, Plane};
use evanescence_core::monte_carlo::{MonteCarlo, Quality};
use evanescence_core::orbital::{self, Orbital, Qn, RadialPlot};
use getset::CopyGetters;
use strum::{Display, EnumDiscriminants, EnumIter, IntoEnumIterator};
use yew_state::SharedHandle;

pub(crate) use self::presets::{HybridKind, HybridPreset, QnPreset};

#[allow(clippy::upper_case_acronyms)] // "XY", etc. are not acronyms.
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumIter, Display)]
pub(crate) enum Visualization {
    None,
    #[strum(serialize = "Radial wavefunction")]
    RadialWavefunction,
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

impl Visualization {
    pub(crate) fn is_cross_section(self) -> bool {
        matches!(
            self,
            Self::CrossSectionXY | Self::CrossSectionYZ | Self::CrossSectionZX
        )
    }

    pub(crate) fn is_enabled(self) -> bool {
        !matches!(self, Self::None)
    }
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
            _ => Err(format!("{:?} does not have an associated plane", value)),
        }
    }
}

impl TryFrom<Visualization> for RadialPlot {
    type Error = String;

    fn try_from(value: Visualization) -> Result<Self, Self::Error> {
        match value {
            Visualization::RadialWavefunction => Ok(RadialPlot::Wavefunction),
            Visualization::RadialProbabilityDistribution => Ok(RadialPlot::ProbabilityDistribution),
            _ => Err(format!("{:?} is not a radial plot", value)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct RealSimpleState {
    preset: QnPreset,
    nodes_rad: bool,
    nodes_ang: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct RealState {
    qn: Qn,
    nodes_rad: bool,
    nodes_ang: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct ComplexState {
    qn: Qn,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct HybridState {
    preset: HybridPreset,
    show_silhouettes: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, EnumDiscriminants)]
#[strum_discriminants(vis(pub(crate)), name(Mode), derive(EnumIter))]
enum StateInner {
    RealSimple(RealSimpleState),
    Real(RealState),
    Complex(ComplexState),
    Hybrid(HybridState),
}

#[allow(clippy::clippy::enum_glob_use)] // Convenience.
use StateInner::*;

impl Default for StateInner {
    fn default() -> Self {
        RealSimple(RealSimpleState::default())
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RealSimple => write!(f, "Real (Simple)"),
            Self::Real => write!(f, "Real (Full)"),
            Self::Complex => write!(f, "Complex"),
            Self::Hybrid => write!(f, "Hybrid"),
        }
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
                    preset: QnPreset::from_qn_lossy(state.qn),
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
                    ..Default::default()
                });
            }
            (Complex(state), Mode::RealSimple) => {
                log::info!("Transition from Complex to Real (Simple) is possibly lossy.");
                *self = RealSimple(RealSimpleState {
                    preset: QnPreset::from_qn_lossy(state.qn),
                    ..Default::default()
                });
            }
            (_, Mode::Hybrid) => *self = Hybrid(HybridState::default()),
            (Hybrid(_), Mode::RealSimple) => *self = RealSimple(RealSimpleState::default()),
            (Hybrid(_), Mode::Real) => *self = Real(RealState::default()),
            (Hybrid(_), Mode::Complex) => *self = Complex(ComplexState::default()),
            // Same state, do nothing.
            (state, mode) if Mode::from(state as &_) == mode => {}
            _ => unreachable!("all cases should have been covered"),
        }
    }
}

impl Mode {
    pub(crate) fn is_real_or_simple(self) -> bool {
        matches!(self, Self::RealSimple | Self::Real)
    }

    pub(crate) fn is_real(self) -> bool {
        matches!(self, Self::Real)
    }

    pub(crate) fn is_complex(self) -> bool {
        matches!(self, Self::Complex)
    }

    pub(crate) fn is_hybrid(self) -> bool {
        matches!(self, Self::Hybrid)
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

    pub(crate) fn available_supplements(&self) -> Vec<Visualization> {
        match &self.mode() {
            Mode::RealSimple | Mode::Real => Visualization::iter().collect(),
            Mode::Complex => vec![
                Visualization::None,
                Visualization::RadialWavefunction,
                Visualization::RadialProbabilityDistribution,
                Visualization::CrossSectionXY,
                Visualization::CrossSectionYZ,
                Visualization::CrossSectionZX,
            ],
            Mode::Hybrid => vec![
                Visualization::None,
                Visualization::CrossSectionXY,
                Visualization::CrossSectionYZ,
                Visualization::CrossSectionZX,
            ],
        }
    }

    pub(crate) fn is_new_orbital(&self, other: &Self) -> bool {
        match (self.mode(), other.mode()) {
            (Mode::Hybrid, Mode::Hybrid) => self.hybrid_preset() != other.hybrid_preset(),
            // Both `RealSimple` or both `Real`:
            (m1, m2) if m1 == m2 => self.qn() != other.qn(),
            (Mode::RealSimple, Mode::Real) | (Mode::Real, Mode::RealSimple) => {
                self.qn() != other.qn()
            }
            // Different modes:
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

    pub(crate) fn qn(&self) -> &Qn {
        match &self.state {
            RealSimple(state) => state.preset.into(),
            Real(state) => &state.qn,
            Complex(state) => &state.qn,
            Hybrid(_) => panic!("hybrid orbital does not have a `qn`"),
        }
    }

    pub(crate) fn qn_mut(&mut self) -> &mut Qn {
        match &mut self.state {
            RealSimple(_) => panic!("simple mode does not allow setting of arbitrary `qn`s"),
            Real(state) => &mut state.qn,
            Complex(state) => &mut state.qn,
            Hybrid(_) => panic!("hybrid orbital does not have a `qn`"),
        }
    }

    pub(crate) fn qn_preset(&self) -> QnPreset {
        match &self.state {
            RealSimple(state) => state.preset,
            _ => panic!("{:?} does not have a `qn` preset", self.mode()),
        }
    }

    pub(crate) fn nodes_rad(&self) -> bool {
        match &self.state {
            RealSimple(state) => state.nodes_rad,
            Real(state) => state.nodes_rad,
            Complex(_) | Hybrid(_) => false,
        }
    }

    pub(crate) fn nodes_ang(&self) -> bool {
        match &self.state {
            RealSimple(state) => state.nodes_ang,
            Real(state) => state.nodes_ang,
            Complex(_) | Hybrid(_) => false,
        }
    }

    pub(crate) fn hybrid_kind(&self) -> &'static HybridKind {
        match &self.state {
            Hybrid(state) => state.preset.into(),
            _ => panic!("{:?} does not have a `hybrid_kind`", self.mode()),
        }
    }

    pub(crate) fn hybrid_preset(&self) -> HybridPreset {
        match &self.state {
            Hybrid(state) => state.preset,
            _ => panic!("{:?} does not have a `hybrid` preset", self.mode()),
        }
    }

    pub(crate) fn silhouettes(&self) -> bool {
        match &self.state {
            Hybrid(state) => state.show_silhouettes,
            _ => false,
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

    pub(crate) fn set_qn_preset(&mut self, preset: QnPreset) {
        match &mut self.state {
            RealSimple(state) => {
                state.preset = preset;
            }
            _ => panic!("{:?} does not have a `qn` preset", self.mode()),
        }
    }

    pub(crate) fn set_nodes_rad(&mut self, visibility: bool) {
        match &mut self.state {
            RealSimple(state) => state.nodes_rad = visibility,
            Real(state) => state.nodes_rad = visibility,
            Complex(_) | Hybrid(_) => panic!("nodes cannot be viewed in {:?}", self.mode()),
        }
    }

    pub(crate) fn set_nodes_ang(&mut self, visibility: bool) {
        match &mut self.state {
            RealSimple(state) => state.nodes_ang = visibility,
            Real(state) => state.nodes_ang = visibility,
            Complex(_) | Hybrid(_) => panic!("nodes cannot be viewed in {:?}", self.mode()),
        }
    }

    pub(crate) fn set_hybrid_preset(&mut self, preset: HybridPreset) {
        match &mut self.state {
            Hybrid(state) => {
                state.preset = preset;
            }
            _ => panic!("{:?} does not have a `hybrid` preset", self.mode()),
        }
    }

    pub(crate) fn set_silhouettes(&mut self, show_silhouettes: bool) {
        match &mut self.state {
            Hybrid(state) => state.show_silhouettes = show_silhouettes,
            _ => panic!("symmetry silhouettes do not exist for {:?}", self.mode()),
        }
    }
}

/// Plotting function wrappers.
impl State {
    pub(crate) fn estimate_radius(&self) -> f32 {
        match self.mode() {
            Mode::RealSimple | Mode::Real | Mode::Complex => {
                orbital::Real::estimate_radius(self.qn())
            }
            Mode::Hybrid => {
                orbital::hybrid::Hybrid::estimate_radius(self.hybrid_kind().principal())
            }
        }
    }

    pub(crate) fn monte_carlo_simulate_real(&self) -> ComponentForm<f32> {
        match self.mode() {
            Mode::RealSimple | Mode::Real => {
                orbital::Real::monte_carlo_simulate(self.qn(), self.quality(), true)
            }
            Mode::Hybrid => orbital::Hybrid::monte_carlo_simulate(
                self.hybrid_kind().principal(),
                self.quality(),
                true,
            ),
            Mode::Complex => panic!("Mode::Complex does not produce real values"),
        }
    }

    pub(crate) fn sample_cross_section_real(&self, plane: Plane) -> GridValues<f32> {
        match self.mode() {
            Mode::RealSimple | Mode::Real => {
                orbital::Real::sample_cross_section(self.qn(), plane, self.quality().for_grid())
            }
            Mode::Hybrid => orbital::Hybrid::sample_cross_section(
                self.hybrid_kind().principal(),
                plane,
                self.quality().for_grid(),
            ),
            Mode::Complex => panic!("Mode::Complex does not produce real values"),
        }
    }
}

pub(crate) type StateHandle = SharedHandle<State>;
