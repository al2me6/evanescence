use evanescence_core::monte_carlo::Quality;
use evanescence_core::orbital::quantum_numbers::Lm;
use evanescence_core::orbital::wavefunctions::RealSphericalHarmonic;
use evanescence_core::orbital::{self, Qn};

use strum::IntoEnumIterator;
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew_state::SharedStateComponent;
use yewtil::NeqAssign;

use crate::components::{CheckBox, Dropdown};
use crate::state::{QnPreset, State, StateHandle, Visualization};
use crate::MAX_N;

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

        fn update_preset(state: &mut State, new_preset: QnPreset) {
            if let QnPreset::Preset(qn) = new_preset {
                state.qn = qn;
            }
            state.qn_preset = new_preset;
        }

        let format_l = |l: u32| {
            if let Some(subshell) = orbital::subshell_name(l) {
                format!("{} [{}]", l, subshell)
            } else {
                l.to_string()
            }
        };

        let format_m = |m: i32| {
            if let Some(expression) = RealSphericalHarmonic::linear_combination_expression(
                Lm::new(state.qn.l(), m).unwrap(),
            ) {
                if !expression.is_empty() {
                    return format!("{} [{}]", m, expression);
                }
            };
            m.to_string()
        };

        let qn_preset_picker = html! {
            <tr>
                <td>{ "Select orbital: " }</td>
                <td><Dropdown<QnPreset>
                    id = "preset_picker"
                    onchange = handle.reduce_callback_with(update_preset)
                    options = QnPreset::iter().collect::<Vec<_>>()
                    selected = QnPreset::default()
                /></td>
            </tr>
        };

        let qn_pickers = {
            let l_options: Vec<_> = Qn::enumerate_l_for_n(state.qn.n()).collect();
            let m_options: Vec<_> = Qn::enumerate_m_for_l(state.qn.l()).collect();

            html! {
                <>
                <tr>
                    <td>{"Principal quantum number n:"}</td>
                    <td><Dropdown<u32>
                        id = "n-picker"
                        onchange = handle.reduce_callback_with(|s, n| s.qn.set_n_clamping(n))
                        options = (1..=MAX_N).collect::<Vec<_>>()
                        selected = state.qn.n()
                    /></td>
                </tr>
                <tr>
                    // U+2113 SCRIPT SMALL L.
                    <td>{"Azimuthal quantum number \u{2113}:"}</td>
                    <td><Dropdown<u32>
                        id = "l-picker"
                        onchange = handle.reduce_callback_with(|s, l| s.qn.set_l_clamping(l))
                        options = l_options
                        custom_display = l_options.iter().map(|&l| format_l(l)).collect::<Vec<_>>()
                        selected = state.qn.l()
                    /></td>
                </tr>
                <tr>
                    <td>{"Magnetic quantum number m:"}</td>
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
        };

        html! {
            <div id = "controls">
                <table>
                    { qn_preset_picker }
                    { if state.qn_preset == QnPreset::Custom { qn_pickers } else { html! {} }}
                    <tr>
                        <td>{"Render quality:"}</td>
                        <td><Dropdown<Quality>
                            id = "quality-picker"
                            onchange = handle.reduce_callback_with(|s, qual| s.quality = qual)
                            options = Quality::iter().collect::<Vec<_>>()
                            selected = state.quality
                        /></td>
                    </tr>
                    <tr>
                        <td/>
                        <td><CheckBox
                            id = "radial-nodes-toggle",
                            onchange = handle.reduce_callback_with(|s, vis| s.nodes_show_radial = vis)
                            initial_state = self.handle.state().nodes_show_radial
                            label = "Show radial nodes"
                        /></td>
                    </tr>
                    <tr>
                        <td/>
                        <td><CheckBox
                            id = "angular-nodes-toggle",
                            onchange = handle.reduce_callback_with(|s, vis| s.nodes_show_angular = vis)
                            initial_state = self.handle.state().nodes_show_angular
                            label = "Show angular nodes"
                        /></td>
                    </tr>
                    <tr>
                        <td>{"Show supplemental visualization:"}</td>
                        <td><Dropdown<Visualization>
                            id = "supplement-picker"
                            onchange = handle.reduce_callback_with(|s, ext| s.extra_visualization = ext)
                            options = Visualization::iter().collect::<Vec<_>>()
                            selected = state.extra_visualization
                        /></td>
                    </tr>
                </table>
            </div>
        }
    }
}

pub(crate) type Controls = SharedStateComponent<ControlsImpl>;
