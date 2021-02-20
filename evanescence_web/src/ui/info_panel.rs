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

        fn node_pluralize(n: u32) -> &'static str {
            if n == 1 {
                "node"
            } else {
                "nodes"
            }
        }

        let description = match state.mode() {
            Mode::RealSimple | Mode::Real => {
                let num_radial_nodes = orbital::Real::num_radial_nodes(state.qn());
                let num_angular_nodes = orbital::Real::num_angular_nodes(state.qn());
                let subshell_name =
                    orbital::subshell_name(state.qn().l()).expect("failed to get subshell name");
                html! {
                    <p>
                    {"Viewing orbital "}
                    <RawSpan inner_html = orbital::Real::name(state.qn()) />
                    { format!(
                        ", which is {} {} orbital with {} radial {} and {} angular {}.",
                        // English is hard.
                        if "sfhi".contains(subshell_name) {
                            "an"
                        } else {
                            "a"
                        },
                        subshell_name,
                        num_radial_nodes,
                        node_pluralize(num_radial_nodes),
                        num_angular_nodes,
                        node_pluralize(num_angular_nodes),
                    ) }
                    </p>
                }
            }
            Mode::Complex => html! {
                <p>
                    {"Viewing orbital "}
                    <RawSpan inner_html = orbital::Complex::name(state.qn()) />
                    { "." }
                </p>
            },
            Mode::Hybrid => {
                html! {
                    <>
                    <p>
                        {"Viewing " }
                        { state.hybrid_kind().kind() }
                        { "-hybridized orbital formed by the linear combination " }
                        <RawSpan inner_html = state.hybrid_kind().principal().expression() />
                        { "." }
                    </p>
                    <p>
                        { "There are " }
                        { state.hybrid_kind().count() }
                        { " " }
                        { state.hybrid_kind().kind() }
                        { " orbitals with " }
                        { state.hybrid_kind().symmetry() }
                        { " symmetry. The other "}
                        { state.hybrid_kind().kind() }
                        { " orbitals (which can be drawn by enabling \"Show symmetry\") are formed from the following linear combinations:" }
                    </p>
                    <ul>
                        { for state.hybrid_kind().rotations().iter().map(|lc| html! {
                            <li><RawSpan inner_html = lc.expression() /></li>
                        })}
                    </ul>
                    </>
                }
            }
        };

        html! {
            <div id = "info-panel">
                <h3>{"Orbital Information"}</h3>
                { description }
                <p>
                { format!("Visualized using {} points.", state.quality() as usize) }
                </p>
            </div>
        }
    }
}

pub(crate) type InfoPanel = SharedStateComponent<InfoPanelImpl>;
