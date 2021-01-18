use evanescence_core::monte_carlo::Quality;
use evanescence_core::orbital::Qn;
use strum::IntoEnumIterator;
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew_state::SharedStateComponent;
use yewtil::NeqAssign;

use crate::components::{CheckBox, Dropdown};
use crate::StateHandle;
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

        let set_n = handle.reduce_callback_with(|s, n| s.qn.set_n_clamping(n));
        let set_l = handle.reduce_callback_with(|s, l| s.qn.set_l_clamping(l));
        let set_m = handle.reduce_callback_with(|s, m| s.qn.set_m(m));
        let set_quality = handle.reduce_callback_with(|s, qual| s.quality = qual);
        let set_nodes_visibility = handle.reduce_callback_with(|s, vis| s.nodes_visibility = vis);

        let state = handle.state();

        html! {
            <div id = "controls">
                <table>
                    <tr>
                        <td>{"Principal quantum number (n):"}</td>
                        <td><Dropdown<u32>
                            id = "n-picker"
                            onchange = set_n,
                            options = (1..=MAX_N).collect::<Vec<_>>()
                            selected = state.qn.n()
                        /></td>
                    </tr>
                    <tr>
                        <td>{"Azimuthal quantum number (l):"}</td>
                        <td><Dropdown<u32>
                            id = "l-picker"
                            onchange = set_l,
                            options = Qn::enumerate_l_for_n(state.qn.n()).collect::<Vec<_>>(),
                            selected = state.qn.l()
                        /></td>
                    </tr>
                    <tr>
                        <td>{"Magnetic quantum number (m):"}</td>
                        <td><Dropdown<i32>
                            id = "m-picker"
                            onchange = set_m,
                            options = Qn::enumerate_m_for_l(state.qn.l()).collect::<Vec<_>>(),
                            selected = state.qn.m()
                        /></td>
                    </tr>
                    <tr>
                        <td>{"Render quality:"}</td>
                        <td><Dropdown<Quality>
                            id = "quality-picker"
                            onchange = set_quality,
                            options = Quality::iter().collect::<Vec<_>>()
                            selected = state.quality
                        /></td>
                    </tr>
                </table>
                <CheckBox
                    id = "nodes-toggle",
                    onchange = set_nodes_visibility,
                    initial_state = self.handle.state().nodes_visibility,
                    label = "Display nodal surfaces"
                />
            </div>
        }
    }
}

pub(crate) type Controls = SharedStateComponent<ControlsImpl>;
