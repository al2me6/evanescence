use std::convert::TryFrom;
use std::default::default;
use std::fmt;

use evanescence_core::geometry::{ComponentForm, GridValues, Plane};
use evanescence_core::monte_carlo::{MonteCarlo, Quality};
use evanescence_core::numerics::EvaluateBounded;
use evanescence_core::orbital::atomic::RadialPlot;
use evanescence_core::orbital::hybrid::Kind;
use evanescence_core::orbital::molecular::Lcao;
use evanescence_core::orbital::{self, Orbital, ProbabilityDensity, Qn};
use getset::CopyGetters;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumDiscriminants, EnumIter, IntoEnumIterator};
use yewdux::prelude::*;

use crate::plotters;
use crate::presets::{HybridPreset, MoPreset, ProtoDiatomicLcao, QnPreset};

#[allow(clippy::upper_case_acronyms)] // "XY", etc. are not acronyms.
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumIter, Display, Serialize, Deserialize)]
pub enum Visualization {
    None,
    #[strum(serialize = "Radial wavefunction")]
    RadialWavefunction,
    #[strum(serialize = "Radial probability distribution")]
    RadialProbabilityDistribution,
    #[strum(serialize = "XY-plane wavefunction")]
    WavefunctionXY,
    #[strum(serialize = "YZ-plane wavefunction")]
    WavefunctionYZ,
    #[strum(serialize = "ZX-plane wavefunction")]
    WavefunctionZX,
    #[strum(serialize = "XY-plane probability density")]
    ProbabilityDensityXY,
    #[strum(serialize = "YZ-plane probability density")]
    ProbabilityDensityYZ,
    #[strum(serialize = "ZX-plane probability density")]
    ProbabilityDensityZX,
    #[strum(serialize = "3D isosurface")]
    Isosurface3D,
}

impl Visualization {
    pub fn is_cross_section(self) -> bool {
        matches!(
            self,
            Self::WavefunctionXY
                | Self::WavefunctionYZ
                | Self::WavefunctionZX
                | Self::ProbabilityDensityXY
                | Self::ProbabilityDensityYZ
                | Self::ProbabilityDensityZX
        )
    }

    pub fn is_radial(self) -> bool {
        matches!(
            self,
            Self::RadialWavefunction | Self::RadialProbabilityDistribution
        )
    }

    pub fn is_enabled(self) -> bool {
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
            Visualization::WavefunctionXY | Visualization::ProbabilityDensityXY => Ok(Plane::XY),
            Visualization::WavefunctionYZ | Visualization::ProbabilityDensityYZ => Ok(Plane::YZ),
            Visualization::WavefunctionZX | Visualization::ProbabilityDensityZX => Ok(Plane::ZX),
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
struct RealSimpleState {
    preset: QnPreset,
    nodes_rad: bool,
    nodes_ang: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
struct RealState {
    qn: Qn,
    nodes_rad: bool,
    nodes_ang: bool,
    instant_apply: bool,
}

impl Default for RealState {
    fn default() -> Self {
        Self {
            qn: default(),
            nodes_rad: default(),
            nodes_ang: default(),
            instant_apply: true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
struct ComplexState {
    qn: Qn,
    instant_apply: bool,
}

impl Default for ComplexState {
    fn default() -> Self {
        Self {
            qn: default(),
            instant_apply: true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
struct HybridState {
    preset: HybridPreset,
    silhouettes: bool,
    nodes: bool,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
struct MoState {
    preset: MoPreset,
    nodes: bool,
    separation: f32,
}

impl std::cmp::Eq for MoState {} // FIXME: ew.

impl Default for MoState {
    fn default() -> Self {
        Self {
            preset: Default::default(),
            nodes: Default::default(),
            separation: 1.398,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, EnumDiscriminants, Serialize, Deserialize)]
#[strum_discriminants(vis(pub), name(Mode), derive(EnumIter))]
enum StateInner {
    RealSimple(RealSimpleState),
    Real(RealState),
    Complex(ComplexState),
    Hybrid(HybridState),
    Mo(MoState),
}

impl Mode {
    pub fn is_real_or_simple(self) -> bool {
        matches!(self, Self::RealSimple | Self::Real)
    }

    pub fn is_real(self) -> bool {
        matches!(self, Self::Real)
    }

    pub fn is_complex(self) -> bool {
        matches!(self, Self::Complex)
    }

    pub fn is_hybrid(self) -> bool {
        matches!(self, Self::Hybrid)
    }

    pub fn is_mo(self) -> bool {
        matches!(self, Self::Mo)
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RealSimple => write!(f, "Real (Simple)"),
            Self::Real => write!(f, "Real (Full)"),
            Self::Complex => write!(f, "Complex"),
            Self::Hybrid => write!(f, "Hybrid"),
            Self::Mo => write!(f, "MO"),
        }
    }
}

#[allow(clippy::enum_glob_use)] // Convenience.
use StateInner::*;

impl Default for StateInner {
    fn default() -> Self {
        RealSimple(RealSimpleState::default())
    }
}

impl StateInner {
    fn transition(&mut self, mode: Mode) {
        match (&self, mode) {
            // To `RealSimple`:
            (Real(state), Mode::RealSimple) => {
                *self = RealSimple(RealSimpleState {
                    preset: QnPreset::from_qn_lossy(state.qn),
                    nodes_rad: state.nodes_rad,
                    nodes_ang: state.nodes_ang,
                });
            }
            (Complex(state), Mode::RealSimple) => {
                *self = RealSimple(RealSimpleState {
                    preset: QnPreset::from_qn_lossy(state.qn),
                    ..default()
                });
            }
            (Hybrid(state), Mode::RealSimple) => {
                *self = RealSimple(RealSimpleState {
                    nodes_rad: state.nodes,
                    nodes_ang: state.nodes,
                    ..default()
                });
            }
            (Mo(_), Mode::RealSimple) => *self = RealSimple(default()),

            // To `Real`:
            (RealSimple(state), Mode::Real) => {
                *self = Real(RealState {
                    qn: state.preset.into(),
                    nodes_rad: state.nodes_rad,
                    nodes_ang: state.nodes_ang,
                    ..default()
                });
            }
            (Complex(state), Mode::Real) => {
                *self = Real(RealState {
                    qn: state.qn,
                    instant_apply: state.instant_apply,
                    ..default()
                });
            }
            (Hybrid(state), Mode::Real) => {
                *self = Real(RealState {
                    nodes_rad: state.nodes,
                    nodes_ang: state.nodes,
                    ..default()
                });
            }
            (Mo(_), Mode::Real) => *self = Real(default()),

            // To `Complex`:
            (RealSimple(state), Mode::Complex) => {
                *self = Complex(ComplexState {
                    qn: state.preset.into(),
                    ..default()
                });
            }
            (Real(state), Mode::Complex) => {
                *self = Complex(ComplexState {
                    qn: state.qn,
                    instant_apply: state.instant_apply,
                });
            }
            (Hybrid(_) | Mo(_), Mode::Complex) => *self = Complex(default()),

            // To `Hybrid`:
            (RealSimple(state), Mode::Hybrid) => {
                *self = Hybrid(HybridState {
                    nodes: state.nodes_rad || state.nodes_ang,
                    ..default()
                });
            }
            (Real(state), Mode::Hybrid) => {
                *self = Hybrid(HybridState {
                    nodes: state.nodes_rad || state.nodes_ang,
                    ..default()
                });
            }
            (Complex(_) | Mo(_), Mode::Hybrid) => *self = Hybrid(default()),

            // To `Mo`:
            (RealSimple(_) | Real(_) | Complex(_) | Hybrid(_), Mode::Mo) => {
                *self = Mo(default());
            }

            // Same state, do nothing:
            (state, mode) if Mode::from(state as &_) == mode => {}
            _ => unreachable!("all cases should have been covered"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, CopyGetters, Default, Serialize, Deserialize)]
pub struct State {
    state: StateInner,
    #[getset(get_copy = "pub")]
    quality: Quality,
    supplement: Visualization,
}

/// Queries.
impl State {
    pub fn mode(&self) -> Mode {
        (&self.state).into()
    }

    pub fn available_supplements(&self) -> Vec<Visualization> {
        match &self.mode() {
            Mode::RealSimple | Mode::Real => Visualization::iter().collect(),
            Mode::Complex => vec![
                Visualization::None,
                Visualization::RadialWavefunction,
                Visualization::RadialProbabilityDistribution,
                Visualization::WavefunctionXY,
                Visualization::WavefunctionYZ,
                Visualization::WavefunctionZX,
                Visualization::ProbabilityDensityXY,
                Visualization::ProbabilityDensityYZ,
                Visualization::ProbabilityDensityZX,
            ],
            Mode::Hybrid => vec![
                Visualization::None,
                Visualization::WavefunctionXY,
                Visualization::WavefunctionYZ,
                Visualization::WavefunctionZX,
                Visualization::ProbabilityDensityXY,
                Visualization::ProbabilityDensityYZ,
                Visualization::ProbabilityDensityZX,
                Visualization::Isosurface3D,
            ],
            Mode::Mo => vec![
                Visualization::None,
                Visualization::WavefunctionXY,
                Visualization::WavefunctionYZ,
                Visualization::WavefunctionZX,
                Visualization::ProbabilityDensityXY,
                Visualization::ProbabilityDensityYZ,
                Visualization::ProbabilityDensityZX,
            ],
        }
    }

    pub fn is_new_orbital(&self, other: &Self) -> bool {
        match (self.mode(), other.mode()) {
            (Mode::Hybrid, Mode::Hybrid) => self.hybrid_preset() != other.hybrid_preset(),
            (Mode::Mo, Mode::Mo) => self.lcao() != other.lcao(),
            (Mode::RealSimple | Mode::Real, Mode::RealSimple | Mode::Real)
            | (Mode::Complex, Mode::Complex) => self.qn() != other.qn(),
            // Different modes:
            _ => true,
        }
    }
}

/// Getters for specific states.
impl State {
    pub fn supplement(&self) -> Visualization {
        assert!(self.available_supplements().contains(&self.supplement));
        self.supplement
    }

    pub fn debug_description(&self) -> String {
        match &self.state {
            RealSimple(_) | Real(_) | Complex(_) => self.qn().to_string_as_wavefunction(),
            Hybrid(_) => self.hybrid_kind().to_string(),
            Mo(_) => orbital::molecular::Molecular::name(&self.lcao()),
        }
    }

    pub fn qn(&self) -> &Qn {
        match &self.state {
            RealSimple(state) => state.preset.into(),
            Real(state) => &state.qn,
            Complex(state) => &state.qn,
            Hybrid(_) | Mo(_) => panic!("hybrid or molecular orbital does not have a `qn`"),
        }
    }

    pub fn qn_preset(&self) -> QnPreset {
        match &self.state {
            RealSimple(state) => state.preset,
            _ => panic!("{:?} does not have a `qn` preset", self.mode()),
        }
    }

    pub fn nodes_rad(&self) -> bool {
        match &self.state {
            RealSimple(state) => state.nodes_rad,
            Real(state) => state.nodes_rad,
            Complex(_) | Hybrid(_) | Mo(_) => false,
        }
    }

    pub fn nodes_ang(&self) -> bool {
        match &self.state {
            RealSimple(state) => state.nodes_ang,
            Real(state) => state.nodes_ang,
            Complex(_) | Hybrid(_) | Mo(_) => false,
        }
    }

    pub fn instant_apply(&self) -> bool {
        match &self.state {
            Real(state) => state.instant_apply,
            Complex(state) => state.instant_apply,
            RealSimple(_) | Hybrid(_) | Mo(_) => {
                panic!("instant-apply does not exist in {:?}", self.mode());
            }
        }
    }

    pub fn hybrid_kind(&self) -> &'static Kind {
        match &self.state {
            Hybrid(state) => state.preset.into(),
            _ => panic!("{:?} does not have a `hybrid_kind`", self.mode()),
        }
    }

    pub fn hybrid_preset(&self) -> HybridPreset {
        match &self.state {
            Hybrid(state) => state.preset,
            _ => panic!("{:?} does not have a `hybrid` preset", self.mode()),
        }
    }

    pub fn silhouettes(&self) -> bool {
        match &self.state {
            Hybrid(state) => state.silhouettes,
            _ => false,
        }
    }

    pub fn nodes(&self) -> bool {
        match &self.state {
            Hybrid(state) => state.nodes,
            Mo(state) => state.nodes,
            _ => false,
        }
    }

    pub fn lcao(&self) -> Lcao {
        match &self.state {
            Mo(state) => {
                <&'_ ProtoDiatomicLcao>::from(state.preset).with_separation(self.separation())
            }
            _ => panic!("{:?} does not have an `lcao`", self.mode()),
        }
    }

    pub fn mo_preset(&self) -> MoPreset {
        match &self.state {
            Mo(state) => state.preset,
            _ => panic!("{:?} does not have a `mo` preset", self.mode()),
        }
    }

    pub fn separation(&self) -> f32 {
        match &self.state {
            Mo(state) => state.separation,
            _ => panic!(
                "{:?} does not have an interatomic `separation`",
                self.mode()
            ),
        }
    }

    pub fn isosurface_cutoff(&self) -> f32 {
        match self.mode() {
            Mode::Real | Mode::RealSimple => plotters::isosurface_cutoff_heuristic_real(self.qn()),
            Mode::Hybrid => plotters::isosurface_cutoff_heuristic_hybrid(self.hybrid_kind()),
            Mode::Complex => panic!("isosurface not available in `Complex` mode"),
            Mode::Mo => todo!("isosurface not available in `Mo` mode"),
        }
    }
}

/// Setters for specific states.
impl State {
    pub fn set_mode(&mut self, mode: Mode) {
        self.state.transition(mode);
        if !self.available_supplements().contains(&self.supplement) {
            self.supplement = Visualization::None;
        }
    }

    pub fn set_quality(&mut self, quality: Quality) {
        self.quality = quality;
    }

    pub fn set_supplement(&mut self, supplement: Visualization) {
        assert!(self.available_supplements().contains(&supplement));
        self.supplement = supplement;
    }

    pub fn set_qn_preset(&mut self, preset: QnPreset) {
        match &mut self.state {
            RealSimple(state) => {
                state.preset = preset;
            }
            _ => panic!("{:?} does not have a `qn` preset", self.mode()),
        }
    }

    pub fn set_nodes_rad(&mut self, visibility: bool) {
        match &mut self.state {
            RealSimple(state) => state.nodes_rad = visibility,
            Real(state) => state.nodes_rad = visibility,
            Complex(_) | Hybrid(_) | Mo(_) => {
                panic!("radial nodes cannot be viewed in {:?}", self.mode());
            }
        }
    }

    pub fn set_nodes_ang(&mut self, visibility: bool) {
        match &mut self.state {
            RealSimple(state) => state.nodes_ang = visibility,
            Real(state) => state.nodes_ang = visibility,
            Complex(_) | Hybrid(_) | Mo(_) => {
                panic!("angular nodes cannot be viewed in {:?}", self.mode());
            }
        }
    }

    pub fn set_qn(&mut self, qn: Qn) {
        match &mut self.state {
            Real(state) => state.qn = qn,
            Complex(state) => state.qn = qn,
            RealSimple(_) | Hybrid(_) | Mo(_) => {
                panic!("`qn` cannot be set for {:?}", self.mode());
            }
        }
    }

    pub fn set_instant_apply(&mut self, instant: bool) {
        match &mut self.state {
            Real(state) => state.instant_apply = instant,
            Complex(state) => state.instant_apply = instant,
            RealSimple(_) | Hybrid(_) | Mo(_) => {
                panic!("instant-apply cannot be set for {:?}", self.mode());
            }
        }
    }

    pub fn set_hybrid_preset(&mut self, preset: HybridPreset) {
        match &mut self.state {
            Hybrid(state) => {
                state.preset = preset;
            }
            _ => panic!("{:?} does not have a `hybrid` preset", self.mode()),
        }
    }

    pub fn set_silhouettes(&mut self, silhouettes: bool) {
        match &mut self.state {
            Hybrid(state) => state.silhouettes = silhouettes,
            _ => panic!("symmetry silhouettes do not exist for {:?}", self.mode()),
        }
    }

    pub fn set_nodes(&mut self, nodes: bool) {
        match &mut self.state {
            Hybrid(state) => state.nodes = nodes,
            Mo(state) => state.nodes = nodes,
            _ => panic!("nodes cannot be set for {:?}", self.mode()),
        }
    }

    pub fn set_mo_preset(&mut self, preset: MoPreset) {
        match &mut self.state {
            Mo(state) => {
                state.preset = preset;
            }
            _ => panic!("{:?} does not have an `mo` preset", self.mode()),
        }
    }

    pub fn set_separation(&mut self, separation: f32) {
        match &mut self.state {
            Mo(state) => {
                state.separation = separation;
            }
            _ => panic!("cannot set separation for {:?}", self.mode()),
        }
    }
}

/// Plotting function wrappers.
impl State {
    pub fn bound(&self) -> f32 {
        match self.mode() {
            Mode::RealSimple | Mode::Real | Mode::Complex => orbital::Real1::bound(self.qn()),
            Mode::Hybrid => orbital::hybrid::Hybrid::bound(self.hybrid_kind().archetype()),
            Mode::Mo => orbital::molecular::Molecular::bound(&self.lcao()),
        }
    }

    pub fn monte_carlo_simulate_real(&self) -> ComponentForm<f32> {
        match self.mode() {
            Mode::RealSimple | Mode::Real => {
                orbital::Real1::monte_carlo_simulate(self.qn(), self.quality(), true)
            }
            Mode::Hybrid => orbital::hybrid::Hybrid::monte_carlo_simulate(
                self.hybrid_kind().archetype(),
                self.quality(),
                true,
            ),
            Mode::Complex => panic!("Mode::Complex does not produce real values"),
            Mode::Mo => orbital::molecular::Molecular::monte_carlo_simulate(
                &self.lcao(),
                self.quality(),
                true,
            ),
        }
    }

    pub fn sample_plane_real(&self, plane: Plane) -> GridValues<f32> {
        match self.mode() {
            Mode::RealSimple | Mode::Real => {
                orbital::Real1::sample_plane(self.qn(), plane, self.quality().for_grid())
            }
            Mode::Hybrid => orbital::hybrid::Hybrid::sample_plane(
                self.hybrid_kind().archetype(),
                plane,
                self.quality().for_grid(),
            ),
            Mode::Complex => panic!("Mode::Complex does not produce real values"),
            Mode::Mo => orbital::molecular::Molecular::sample_plane(
                &self.lcao(),
                plane,
                self.quality().for_grid(),
            ),
        }
    }

    pub fn sample_plane_prob_density(&self, plane: Plane) -> GridValues<f32> {
        match self.mode() {
            Mode::RealSimple | Mode::Real => ProbabilityDensity::<orbital::Real1>::sample_plane(
                self.qn(),
                plane,
                self.quality().for_grid(),
            ),
            Mode::Complex => ProbabilityDensity::<orbital::Complex>::sample_plane(
                self.qn(),
                plane,
                self.quality().for_grid(),
            ),
            Mode::Hybrid => ProbabilityDensity::<orbital::hybrid::Hybrid>::sample_plane(
                self.hybrid_kind().archetype(),
                plane,
                self.quality().for_grid(),
            ),
            Mode::Mo => ProbabilityDensity::<orbital::molecular::Molecular>::sample_plane(
                &self.lcao(),
                plane,
                self.quality().for_grid(),
            ),
        }
    }
}

impl Persistent for State {
    fn area() -> Area {
        Area::Session
    }
}

#[cfg(feature = "persistent")]
pub type AppDispatch = DispatchProps<PersistentStore<State>>;
#[cfg(not(feature = "persistent"))]
pub type AppDispatch = DispatchProps<BasicStore<State>>;
