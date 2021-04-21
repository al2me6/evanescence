use evanescence_core::orbital::{self, Orbital};
use yew::prelude::*;
use yewdux::prelude::*;
use yewtil::NeqAssign;

use crate::components::raw::RawSpan;
use crate::state::{AppDispatch, Mode};
use crate::utils;

pub(crate) struct InfoPanelImpl {
    dispatch: AppDispatch,
}

impl Component for InfoPanelImpl {
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
        fn node_pluralize(n: u32) -> &'static str {
            if n == 1 {
                "node"
            } else {
                "nodes"
            }
        }

        let state = self.dispatch.state();

        let description = match state.mode() {
            Mode::RealSimple | Mode::Real => {
                let num_radial_nodes = orbital::Real::num_radial_nodes(state.qn());
                let num_angular_nodes = orbital::Real::num_angular_nodes(state.qn());
                let subshell_name =
                    orbital::subshell_name(state.qn().l()).expect("failed to get subshell name");
                html! {
                    <p>
                        { "Viewing orbital " }
                        <RawSpan
                            inner_html = utils::italicize_orbital_name(orbital::Real::name(state.qn()))
                        />
                        { ", which is " }
                        { if "sfhi".contains(subshell_name) { "an " } else { "a " } } // English is hard.
                        <i>{ subshell_name }</i>
                        { format!(
                            " orbital with {} radial {} and {} angular {}.",
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
                let kind = state.hybrid_kind();
                html! {
                    <>
                    <p>
                        {"Viewing " }
                        { kind.kind() }
                        { "-hybridized orbital formed by the linear combination " }
                        <RawSpan inner_html = kind.principal().expression() />
                        { "." }
                    </p>
                    <p>
                        { "There are " }
                        { kind.count() }
                        { " " }
                        { kind.kind() }
                        { " orbitals with " }
                        { kind.symmetry() }
                        { " symmetry. The other " }
                        { kind.kind() }
                        { " orbitals (which can be drawn by enabling \"Show symmetry\") are formed from the following linear combinations:" }
                    </p>
                    <ul>
                        { for kind.rotations().iter().map(|lc| html! {
                            <li><RawSpan inner_html = lc.expression() /></li>
                        }) }
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

pub(crate) type InfoPanel = WithDispatch<InfoPanelImpl>;
