use std::cell::RefCell;
use std::rc::Rc;

use evanescence_core::monte_carlo::Quality;
use evanescence_core::orbital::Qn;
use strum::IntoEnumIterator;
use yew::{html, Callback, Component, ComponentLink, Html, Properties, ShouldRender};
use yewtil::NeqAssign;

use crate::components::Dropdown;
use crate::{AppState, MAX_N};

pub(crate) struct Controls {
    link: ComponentLink<Self>,
    props: ControlsProps,
}

#[derive(Clone, PartialEq, Properties)]
pub(crate) struct ControlsProps {
    pub(crate) onchange: Callback<()>,
    pub(crate) state: Rc<RefCell<AppState>>,
}

pub(crate) enum ControlsMsg {
    N(u32),
    L(u32),
    M(i32),
    Quality(Quality),
}

impl Component for Controls {
    type Message = ControlsMsg;
    type Properties = ControlsProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        {
            let mut state = self.props.state.borrow_mut();
            match msg {
                ControlsMsg::N(n) => {
                    state.qn.set_n_clamping(n);
                }
                ControlsMsg::L(l) => {
                    state.qn.set_l_clamping(l);
                }
                ControlsMsg::M(m) => {
                    state.qn.set_m(m);
                }
                ControlsMsg::Quality(quality) => {
                    state.quality = quality;
                }
            }
        }
        self.props.onchange.emit(());
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let state = self.props.state.borrow();
        html! {
            <div id = "controls">
                <table>
                    <tr>
                        <td>{"Principal quantum number (n):"}</td>
                        <td><Dropdown<u32>
                            id = "n-picker"
                            onchange = self.link.callback(|selected| ControlsMsg::N(selected))
                            options = (1..=MAX_N).collect::<Vec<_>>()
                            selected = state.qn.n()
                        /></td>
                    </tr>
                    <tr>
                        <td>{"Azimuthal quantum number (l):"}</td>
                        <td><Dropdown<u32>
                            id = "l-picker"
                            onchange = self.link.callback(|selected| ControlsMsg::L(selected))
                            options = Qn::enumerate_l_for_n(state.qn.n()).collect::<Vec<_>>(),
                            selected = state.qn.l()
                        /></td>
                    </tr>
                    <tr>
                        <td>{"Magnetic quantum number (m):"}</td>
                        <td><Dropdown<i32>
                            id = "m-picker"
                            onchange = self.link.callback(|selected| ControlsMsg::M(selected))
                            options = Qn::enumerate_m_for_l(state.qn.l()).collect::<Vec<_>>(),
                            selected = state.qn.m()
                        /></td>
                    </tr>
                    <tr>
                        <td>{"Render quality:"}</td>
                        <td><Dropdown<Quality>
                            id = "quality-picker"
                            onchange = self.link.callback(|selected| ControlsMsg::Quality(selected))
                            options = Quality::iter().collect::<Vec<_>>()
                            selected = state.quality
                        /></td>
                    </tr>
                </table>
            </div>
        }
    }
}
