use evanescence_core::monte_carlo::Quality;
use evanescence_core::orbital::quantum_numbers::Lm;
use evanescence_core::orbital::wavefunctions::RealSphericalHarmonic;
use evanescence_core::orbital::{self, Qn};

use strum::IntoEnumIterator;
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew_state::SharedStateComponent;
use yewtil::NeqAssign;

use crate::components::{CheckBox, Dropdown, Tooltip};
use crate::descriptions::DESC;
use crate::state::{Mode, QnPreset, State, StateHandle, Visualization};
use crate::utils::fire_resize_event;
use crate::MAX_N;

fn td_tooltip(text: &str, tooltip: &str) -> Html {
    html! {
        <td><Tooltip text = text tooltip = tooltip /></td>
    }
}

pub(crate) struct ControlsImpl {
    handle: StateHandle,
}

impl Component for ControlsImpl {
    type Message = ();
    type Properties = StateHandle;

    fn create(handle: StateHandle, _link: ComponentLink<Self>) -> Self {
        Self { handle }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, handle: StateHandle) -> ShouldRender {
        self.handle.neq_assign(handle)
    }

    fn view(&self) -> Html {
        let handle = &self.handle;
        let state = handle.state();

        html! {
            <div id = "controls">
                <table>
                    <tr>
                        <td>{ "Select visualization type" }</td>
                        <td><Dropdown<Mode>
                        onchange = handle.reduce_callback_with(State::set_mode)
                        options = Mode::iter().collect::<Vec<_>>()
                        selected = state.mode()
                        /></td>
                    </tr>
                    { if state.mode().is_real() { self.real_mode_controls() } else { self.qn_pickers() }}
                    <tr>
                        { td_tooltip("Show supplemental visualization:", DESC.supplement) }
                        <td><Dropdown<Visualization>
                            id = "supplement-picker"
                            onchange = handle.reduce_callback_with(|s, supp| s.supplement = supp)
                            options = state.available_supplements()
                            selected = state.supplement
                        /></td>
                    </tr>
                    <tr>
                        { td_tooltip("Render quality:", DESC.render_qual) }
                        <td><Dropdown<Quality>
                            id = "quality-picker"
                            onchange = handle.reduce_callback_with(|s, qual| s.quality = qual)
                            options = Quality::iter().collect::<Vec<_>>()
                            selected = state.quality
                        /></td>
                    </tr>
                </table>
            </div>
        }
    }
}

impl ControlsImpl {
    fn real_mode_controls(&self) -> Html {
        let handle = &self.handle;
        let state = handle.state();

        let update_preset = |state: &mut State, new_preset: QnPreset| {
            // Fire resize event to update the size of the supplemental plot when switching
            // between preset and custom modes.
            if state.qn_preset.is_preset() != new_preset.is_preset() {
                fire_resize_event()
            }
            if let QnPreset::Preset(qn) = new_preset {
                state.qn = qn;
            }
            state.qn_preset = new_preset;
        };

        html! {
            <>
            <tr>
                { td_tooltip("Select orbital:", DESC.qn_dropdown) }
                <td><Dropdown<QnPreset>
                    id = "preset_picker"
                    onchange = handle.reduce_callback_with(update_preset)
                    options = QnPreset::iter().collect::<Vec<_>>()
                    selected = state.qn_preset
                /></td>
            </tr>
            { if state.qn_preset == QnPreset::Custom { self.qn_pickers() } else { html! {} }}
            <tr>
                <td/>
                <td><CheckBox
                    id = "radial-nodes-toggle",
                    onchange = handle.reduce_callback_with(|s, vis| s.nodes_show_radial = vis)
                    initial_state = state.nodes_show_radial
                    label = "Show radial nodes"
                    tooltip = DESC.rad_nodes
                /></td>
            </tr>
            <tr>
                <td/>
                <td><CheckBox
                    id = "angular-nodes-toggle",
                    onchange = handle.reduce_callback_with(|s, vis| s.nodes_show_angular = vis)
                    initial_state = state.nodes_show_angular
                    label = "Show angular nodes"
                    tooltip = DESC.ang_nodes
                /></td>
            </tr>
            </>
        }
    }

    fn qn_pickers(&self) -> Html {
        let handle = &self.handle;
        let state = handle.state();

        let l_options: Vec<_> = Qn::enumerate_l_for_n(state.qn.n()).collect();
        let m_options: Vec<_> = Qn::enumerate_m_for_l(state.qn.l()).collect();

        let format_l = |l: u32| match orbital::subshell_name(l) {
            Some(subshell) => format!("{} [{}]", l, subshell),
            None => l.to_string(),
        };

        let format_m = |m: i32| match state.mode() {
            Mode::Real => match RealSphericalHarmonic::linear_combination_expression(
                Lm::new(state.qn.l(), m).unwrap(),
            ) {
                Some(expression) if !expression.is_empty() => format!("{} [ {} ]", m, expression),
                _ => m.to_string(),
            },
            Mode::Complex => m.to_string(),
        };

        html! {
            <>
            <tr>
                { td_tooltip("Principal quantum number n:", DESC.qn_n) }
                <td><Dropdown<u32>
                    id = "n-picker"
                    onchange = handle.reduce_callback_with(|s, n| s.qn.set_n_clamping(n))
                    options = (1..=MAX_N).collect::<Vec<_>>()
                    selected = state.qn.n()
                /></td>
            </tr>
            <tr>
                { td_tooltip("Azimuthal quantum number ℓ:", DESC.qn_l) }
                <td><Dropdown<u32>
                    id = "l-picker"
                    onchange = handle.reduce_callback_with(|s, l| s.qn.set_l_clamping(l))
                    options = l_options
                    custom_display = l_options.iter().map(|&l| format_l(l)).collect::<Vec<_>>()
                    selected = state.qn.l()
                /></td>
            </tr>
            <tr>
                { td_tooltip("Magnetic quantum number m:", DESC.qn_m) }
                <td><Dropdown<i32>
                    id = "m-picker"
                    onchange = handle.reduce_callback_with(|s, m| s.qn.set_m(m))
                    options = m_options
                    custom_display = m_options.iter().map(|&m| format_m(m)).collect::<Vec<_>>()
                    selected = state.qn.m()
                /></td>
            </tr>
            </>
        }
    }
}

pub(crate) type Controls = SharedStateComponent<ControlsImpl>;
