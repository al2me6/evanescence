use evanescence_core::orbital::{self, Orbital};
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew_state::SharedStateComponent;
use yewtil::NeqAssign;

use crate::components::RawSpan;
use crate::state::{Mode, StateHandle};

pub(crate) struct InfoPanelImpl {
    handle: StateHandle,
}

impl Component for InfoPanelImpl {
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
        let state = self.handle.state();
        let num_radial_nodes = orbital::Real::num_radial_nodes(state.qn());
        let num_angular_nodes = orbital::Real::num_angular_nodes(state.qn());
        let subshell_name = orbital::subshell_name(state.qn().l()).unwrap();

        fn node_pluralize(n: u32) -> &'static str {
            if n == 1 {
                "node"
            } else {
                "nodes"
            }
        }

        let orbital_name = match state.mode() {
            Mode::RealSimple | Mode::Real => orbital::Real::name,
            Mode::Complex => orbital::Complex::name,
        };

        html! {
            <div id = "info-panel">
                <h3>{"Orbital Information"}</h3>
                <p>
                    {"Viewing orbital "}
                    <RawSpan inner_html = orbital_name(state.qn()) />
                    { if state.is_real() {
                        format!(
                            ", which is {} {} orbital with {} radial {} and {} angular {}.",
                            // English is hard.
                            if "sfhi".contains(subshell_name) { "an" } else { "a" },
                            subshell_name,
                            num_radial_nodes,
                            node_pluralize(num_radial_nodes),
                            num_angular_nodes,
                            node_pluralize(num_angular_nodes),
                        )
                    } else {
                        ".".to_owned()
                    }}
                </p>
                <p>
                { format!("Visualized using {} points.", state.quality() as usize) }
                </p>
            </div>
        }
    }
}

pub(crate) type InfoPanel = SharedStateComponent<InfoPanelImpl>;
