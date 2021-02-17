use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::f32::consts::{FRAC_1_SQRT_2, SQRT_2};
use std::fmt;

use evanescence_core::{
    geometry::{ComponentForm, GridValues, Plane},
    lc,
    monte_carlo::{MonteCarlo, Quality},
    orbital::wavefunctions::RealSphericalHarmonic,
    orbital::{self, LinearCombination, Orbital, Qn, RadialPlot},
};
use getset::CopyGetters;
use once_cell::sync::Lazy;
use strum::{Display, EnumDiscriminants, EnumIter, IntoEnumIterator};
use yew_state::SharedHandle;

const FRAC_1_SQRT_3: f32 = 0.5773503;
const FRAC_1_SQRT_6: f32 = 0.4082483;
const SQRT_3: f32 = 1.7320508;

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

impl From<QnPreset> for &'static Qn {
    fn from(preset: QnPreset) -> Self {
        &QN_PRESETS[preset.0]
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
        let subscript = RealSphericalHarmonic::expression(&qn.into()).unwrap();
        write!(
            f,
            "{principal}{shell} {subscript}",
            principal = qn.n(),
            shell = orbital::subshell_name(qn.l()).unwrap(),
            subscript = subscript
        )
    }
}

static HYBRIDIZED_PRESETS: Lazy<Vec<LinearCombination>> = Lazy::new(|| {
    vec![
        lc! {
            kind = "sp",
            overall = FRAC_1_SQRT_2,
            (2, 0, 0) => 1.0,
            (2, 1, 0) => 1.0,
        },
        lc! {
            kind = "sp",
            overall = FRAC_1_SQRT_2,
            (2, 0, 0) => 1.0,
            (2, 1, 0) => -1.0,
        },
        lc! {
            kind = "sp²",
            overall = FRAC_1_SQRT_3,
            (2, 0, 0) => 1.0,
            (2, 1, 1) => -SQRT_2,
        },
        lc! {
            kind = "sp²",
            overall = FRAC_1_SQRT_6,
            (2, 0, 0) => SQRT_2,
            (2, 1, 1) => 1.0,
            (2, 1, -1) => SQRT_3,
        },
        lc! {
            kind = "sp²",
            overall = FRAC_1_SQRT_6,
            (2, 0, 0) => SQRT_2,
            (2, 1, 1) => 1.0,
            (2, 1, -1) => -SQRT_3,
        },
        lc! {
            kind = "sp³",
            overall = 0.5,
            (2, 0, 0) => 1.0,
            (2, 1, 1) => 1.0,
            (2, 1, -1) => 1.0,
            (2, 1, 0) => 1.0,
        },
        lc! {
            kind = "sp³",
            overall = 0.5,
            (2, 0, 0) => 1.0,
            (2, 1, 1) => -1.0,
            (2, 1, -1) => -1.0,
            (2, 1, 0) => 1.0,
        },
        lc! {
            kind = "sp³",
            overall = 0.5,
            (2, 0, 0) => 1.0,
            (2, 1, 1) => 1.0,
            (2, 1, -1) => -1.0,
            (2, 1, 0) => -1.0,
        },
        lc! {
            kind = "sp³",
            overall = 0.5,
            (2, 0, 0) => 1.0,
            (2, 1, 1) => -1.0,
            (2, 1, -1) => 1.0,
            (2, 1, 0) => -1.0,
        },
    ]
});

static HYBRIDIZED_DISP_NAMES: Lazy<Vec<String>> = Lazy::new(|| {
    let kinds = HYBRIDIZED_PRESETS
        .iter()
        .map(LinearCombination::kind)
        .collect::<Vec<_>>();
    let has_duplicates = kinds
        .iter()
        .map(|&kind| kinds.iter().filter(|&&k| k == kind).count() > 1)
        .collect::<Vec<_>>();
    let mut counters: HashMap<&String, usize> = HashMap::new();
    kinds
        .into_iter()
        .enumerate()
        .map(|(idx, kind)| {
            if has_duplicates[idx] {
                counters
                    .entry(kind)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
                format!("{} ({})", kind, counters[kind])
            } else {
                kind.clone()
            }
        })
        .collect()
});

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct HybridizedPreset(usize);

impl HybridizedPreset {
    pub(crate) fn iter() -> impl Iterator<Item = Self> {
        (0..HYBRIDIZED_PRESETS.len()).map(Self)
    }
}

impl Default for HybridizedPreset {
    fn default() -> Self {
        Self(0)
    }
}

impl From<HybridizedPreset> for &'static LinearCombination {
    fn from(preset: HybridizedPreset) -> Self {
        &HYBRIDIZED_PRESETS[preset.0]
    }
}

impl fmt::Display for HybridizedPreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", HYBRIDIZED_DISP_NAMES[self.0])
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
struct HybridizedState {
    preset: HybridizedPreset,
}

#[derive(Clone, PartialEq, Eq, Debug, EnumDiscriminants)]
#[strum_discriminants(vis(pub(crate)), name(Mode), derive(EnumIter))]
enum StateInner {
    RealSimple(RealSimpleState),
    Real(RealState),
    Complex(ComplexState),
    Hybridized(HybridizedState),
}

use StateInner::*;

impl Mode {
    pub(crate) fn is_real(&self) -> bool {
        matches!(self, Self::RealSimple | Self::Real)
    }

    pub(crate) fn is_complex(&self) -> bool {
        matches!(self, Self::Complex)
    }

    pub(crate) fn is_hybridized(&self) -> bool {
        matches!(self, Self::Hybridized)
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RealSimple => write!(f, "Real (Simple)"),
            Self::Real => write!(f, "Real (Full)"),
            Self::Complex => write!(f, "Complex"),
            Self::Hybridized => write!(f, "Hybridized"),
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
                    ..Default::default()
                });
            }
            (Complex(state), Mode::RealSimple) => {
                log::info!("Transition from Complex to Real (Simple) is possibly lossy.");
                *self = RealSimple(RealSimpleState {
                    preset: state.qn.try_into().unwrap_or_default(),
                    ..Default::default()
                });
            }
            (_, Mode::Hybridized) => *self = Hybridized(HybridizedState::default()),
            (Hybridized(_), Mode::RealSimple) => *self = RealSimple(RealSimpleState::default()),
            (Hybridized(_), Mode::Real) => *self = Real(RealState::default()),
            (Hybridized(_), Mode::Complex) => *self = Complex(ComplexState::default()),
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

    pub(crate) fn available_supplements(&self) -> Vec<Visualization> {
        match &self.state {
            RealSimple(_) | Real(_) => Visualization::iter().collect(),
            Complex(_) => vec![
                Visualization::None,
                Visualization::RadialWavefunction,
                Visualization::RadialProbabilityDensity,
                Visualization::RadialProbabilityDistribution,
            ],
            Hybridized(_) => vec![
                Visualization::None,
                Visualization::CrossSectionXY,
                Visualization::CrossSectionYZ,
                Visualization::CrossSectionZX,
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
            (Mode::Hybridized, Mode::Hybridized) => {
                self.hybridized_preset() != other.hybridized_preset()
            }
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

    pub(crate) fn qn(&self) -> &Qn {
        match &self.state {
            RealSimple(state) => state.preset.into(),
            Real(state) => &state.qn,
            Complex(state) => &state.qn,
            Hybridized(_) => unreachable!(),
        }
    }

    pub(crate) fn qn_mut(&mut self) -> &mut Qn {
        match &mut self.state {
            RealSimple(_) => unreachable!(),
            Real(state) => &mut state.qn,
            Complex(state) => &mut state.qn,
            Hybridized(_) => unreachable!(),
        }
    }

    pub(crate) fn qn_preset(&self) -> QnPreset {
        match &self.state {
            RealSimple(state) => state.preset,
            _ => unreachable!(),
        }
    }

    pub(crate) fn linear_combination(&self) -> &'static LinearCombination {
        match &self.state {
            Hybridized(state) => state.preset.into(),
            _ => unreachable!(),
        }
    }

    pub(crate) fn hybridized_preset(&self) -> HybridizedPreset {
        match &self.state {
            Hybridized(state) => state.preset,
            _ => unreachable!(),
        }
    }

    pub(crate) fn nodes_rad(&self) -> bool {
        match &self.state {
            RealSimple(state) => state.nodes_rad,
            Real(state) => state.nodes_rad,
            Complex(_) | Hybridized(_) => false,
        }
    }

    pub(crate) fn nodes_ang(&self) -> bool {
        match &self.state {
            RealSimple(state) => state.nodes_ang,
            Real(state) => state.nodes_ang,
            Complex(_) | Hybridized(_) => false,
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
            _ => unreachable!(),
        }
    }

    pub(crate) fn set_nodes_rad(&mut self, visibility: bool) {
        match &mut self.state {
            RealSimple(state) => state.nodes_rad = visibility,
            Real(state) => state.nodes_rad = visibility,
            Complex(_) | Hybridized(_) => unreachable!(),
        }
    }

    pub(crate) fn set_nodes_ang(&mut self, visibility: bool) {
        match &mut self.state {
            RealSimple(state) => state.nodes_ang = visibility,
            Real(state) => state.nodes_ang = visibility,
            Complex(_) | Hybridized(_) => unreachable!(),
        }
    }

    pub(crate) fn set_hybridized_preset(&mut self, preset: HybridizedPreset) {
        match &mut self.state {
            Hybridized(state) => {
                state.preset = preset;
            }
            _ => unreachable!(),
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
            Mode::Hybridized => orbital::Hybridized::estimate_radius(self.linear_combination()),
        }
    }

    pub(crate) fn monte_carlo_simulate_real(&self) -> ComponentForm<f32> {
        match self.mode() {
            Mode::RealSimple | Mode::Real => {
                orbital::Real::monte_carlo_simulate(self.qn(), self.quality())
            }
            Mode::Hybridized => {
                orbital::Hybridized::monte_carlo_simulate(self.linear_combination(), self.quality())
            }
            Mode::Complex => unreachable!(),
        }
    }

    pub(crate) fn sample_cross_section_real(&self, plane: Plane) -> GridValues<f32> {
        match self.mode() {
            Mode::RealSimple | Mode::Real => {
                orbital::Real::sample_cross_section(self.qn(), plane, self.quality().for_grid())
            }
            Mode::Hybridized => orbital::Hybridized::sample_cross_section(
                self.linear_combination(),
                plane,
                self.quality().for_grid(),
            ),
            Mode::Complex => unreachable!(),
        }
    }
}

pub(crate) type StateHandle = SharedHandle<State>;
