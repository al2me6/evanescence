use evanescence_core::monte_carlo::Quality;
use evanescence_core::orbital::atomic::RealSphericalHarmonic;
use evanescence_core::orbital::quantum_numbers::Lm;
use evanescence_core::orbital::{self, Qn};
use itertools::Itertools;
use strum::IntoEnumIterator;
use yew::prelude::*;
use yewdux::prelude::*;
use yewtil::NeqAssign;

use super::descriptions::DESC;
use crate::components::{CheckBox, Dropdown, Tooltip};
use crate::presets::{HybridPreset, QnPreset};
use crate::state::{AppDispatch, Mode, State, Visualization};
use crate::{utils, MAX_N};

fn td_tooltip(text: &'static str, tooltip: &'static str) -> Html {
    html! {
        <td><Tooltip text = text tooltip = tooltip /></td>
    }
}

pub(crate) struct ControlsImpl {
    dispatch: AppDispatch,
}

impl Component for ControlsImpl {
    type Message = ();
    type Properties = AppDispatch;

    fn create(dispatch: AppDispatch, _link: ComponentLink<Self>) -> Self {
        Self { dispatch }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, dispatch: AppDispatch) -> ShouldRender {
        self.dispatch.neq_assign(dispatch)
    }

    fn view(&self) -> Html {
        let dispatch = &self.dispatch;
        let state = dispatch.state();

        let selectors = match state.mode() {
            Mode::RealSimple | Mode::Real => self.real_modes_controls(),
            Mode::Complex => self.qn_pickers(),
            Mode::Hybrid => self.hybrid_picker(),
        };

        html! {
            <div id = "controls">
                <table>
                    { selectors }
                    <tr>
                        { td_tooltip("Show supplemental visualization:", DESC.supplement) }
                        <td><Dropdown<Visualization>
                            id = "supplement-picker"
                            onchange = dispatch.reduce_callback_with(State::set_supplement)
                            options = state.available_supplements()
                            selected = state.supplement()
                        /></td>
                    </tr>
                    <tr>
                        { td_tooltip("Render quality:", DESC.render_qual) }
                        <td><Dropdown<Quality>
                            id = "quality-picker"
                            onchange = dispatch.reduce_callback_with(State::set_quality)
                            options = Quality::iter().collect_vec()
                            custom_display = Quality::iter().map(Quality::to_text).collect_vec()
                            selected = state.quality()
                        /></td>
                    </tr>
                </table>
            </div>
        }
    }
}

impl ControlsImpl {
    fn real_modes_controls(&self) -> Html {
        let dispatch = &self.dispatch;
        let state = dispatch.state();
        assert!(state.mode().is_real_or_simple());

        html! {
            <>
            { if state.mode().is_real() { self.qn_pickers() } else { html! {
                <tr>
                    { td_tooltip("Select orbital:", DESC.qn_dropdown) }
                    <td><Dropdown<QnPreset>
                        id = "preset_picker"
                        onchange = dispatch.reduce_callback_with(State::set_qn_preset)
                        options = QnPreset::iter().collect_vec()
                        selected = state.qn_preset()
                    /></td>
                </tr>
            }} }
            <tr>
                <td/>
                <td><CheckBox
                    id = "radial-nodes-toggle"
                    onchange = dispatch.reduce_callback_with(State::set_nodes_rad)
                    initial_state = state.nodes_rad()
                    label = "Show radial nodes"
                    tooltip = DESC.nodes_rad
                /></td>
            </tr>
            <tr>
                <td/>
                <td><CheckBox
                    id = "angular-nodes-toggle"
                    onchange = dispatch.reduce_callback_with(State::set_nodes_ang)
                    initial_state = state.nodes_ang()
                    label = "Show angular nodes"
                    tooltip = DESC.nodes_ang
                /></td>
            </tr>
            </>
        }
    }

    fn qn_pickers(&self) -> Html {
        let dispatch = &self.dispatch;
        let state = dispatch.state();
        assert!(state.mode().is_real_or_simple() || state.mode().is_complex());

        let l_options = Qn::enumerate_l_for_n(state.qn().n()).unwrap().collect_vec();
        let m_options = Qn::enumerate_m_for_l(state.qn().l()).collect_vec();

        let format_l = |l: u32| match orbital::atomic::subshell_name(l) {
            Some(subshell) => format!("{} [{}]", l, subshell),
            None => l.to_string(),
        };

        let format_m = |m: i32| match state.mode() {
            Mode::RealSimple | Mode::Real => {
                match RealSphericalHarmonic::expression(&Lm::new(state.qn().l(), m).unwrap()) {
                    Some(expression) if !expression.is_empty() => {
                        format!("{} [ {} ]", utils::fmt_replace_minus(m), expression)
                    }
                    _ => utils::fmt_replace_minus(m),
                }
            }
            Mode::Complex => utils::fmt_replace_minus(m),
            Mode::Hybrid => unreachable!(),
        };

        html! {
            <>
            <tr>
                { td_tooltip("Principal quantum number <i>n</i>:", DESC.qn_n) }
                <td><Dropdown<u32>
                    id = "n-picker"
                    onchange = dispatch.reduce_callback_with(|s, n| s.qn_mut().set_n_clamping(n).unwrap())
                    options = (1..=MAX_N).collect_vec()
                    selected = state.qn().n()
                /></td>
            </tr>
            <tr>
                { td_tooltip("Azimuthal quantum number <i>ℓ</i>:", DESC.qn_l) }
                <td><Dropdown<u32>
                    id = "l-picker"
                    onchange = dispatch.reduce_callback_with(|s, l| s.qn_mut().set_l_clamping(l).unwrap())
                    options = l_options
                    custom_display = l_options.iter().map(|&l| format_l(l)).collect_vec()
                    selected = state.qn().l()
                /></td>
            </tr>
            <tr>
                { td_tooltip("Magnetic quantum number <i>m</i>:", DESC.qn_m) }
                <td><Dropdown<i32>
                    id = "m-picker"
                    onchange = dispatch.reduce_callback_with(|s, m| s.qn_mut().set_m(m).unwrap())
                    options = m_options
                    custom_display = m_options.iter().map(|&m| format_m(m)).collect_vec()
                    selected = state.qn().m()
                /></td>
            </tr>
            </>
        }
    }

    fn hybrid_picker(&self) -> Html {
        let dispatch = &self.dispatch;
        let state = dispatch.state();
        assert!(state.mode().is_hybrid());

        html! {
            <>
            <tr>
                { td_tooltip("Select hybridization:", DESC.hybrid_dropdown) }
                <td><Dropdown<HybridPreset>
                    id = "preset_picker"
                    onchange = dispatch.reduce_callback_with(State::set_hybrid_preset)
                    options = HybridPreset::iter().collect_vec()
                    selected = state.hybrid_preset()
                /></td>
            </tr>
            <tr>
                <td/>
                <td><CheckBox
                    id = "show-symmetry-toggle"
                    onchange = dispatch.reduce_callback_with(State::set_silhouettes)
                    initial_state = state.silhouettes()
                    label = "Show symmetry"
                    tooltip = DESC.show_symmetry
                /></td>
            </tr>
            <tr>
                <td/>
                <td><CheckBox
                    id = "hybrid-nodes-toggle"
                    onchange = dispatch.reduce_callback_with(State::set_nodes_hybrid)
                    initial_state = state.nodes_hybrid()
                    label = "Show nodes"
                    tooltip = DESC.nodes_hybrid
                /></td>
            </tr>
            </>
        }
    }
}

pub(crate) type Controls = WithDispatch<ControlsImpl>;
