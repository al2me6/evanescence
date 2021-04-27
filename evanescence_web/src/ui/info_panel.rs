use evanescence_core::orbital::{self, Complex, Orbital, Real};
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
                let num_radial_nodes = Real::num_radial_nodes(state.qn());
                let num_angular_nodes = Real::num_angular_nodes(state.qn());
                let subshell_name =
                    orbital::subshell_name(state.qn().l()).expect("failed to get subshell name");
                html! {
                    <p>
                        { "Viewing orbital " }
                        <RawSpan
                            inner_html = utils::fmt_orbital_name_html(Real::name(state.qn()))
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
                    <RawSpan inner_html = Complex::name(state.qn()) />
                    { "." }
                </p>
            },
            Mode::Hybrid => {
                let kind = state.hybrid_kind();
                let kind_name = html! {
                    <RawSpan inner_html = utils::fmt_orbital_name_html(kind.kind()) />
                };
                html! {
                    <>
                    <p>
                        {"Viewing " }
                        { kind_name.clone() }
                        { "-hybridized orbital formed by the linear combination " }
                        <RawSpan inner_html = utils::fmt_orbital_name_html(
                            kind.principal().expression()
                        ) />
                        { "." }
                    </p>
                    <p>
                        { "There are " }
                        { kind.count() }
                        { " " }
                        { kind_name.clone() }
                        { " orbitals with " }
                        { kind.symmetry() }
                        { " symmetry. The other " }
                        { kind_name.clone() }
                        { " orbitals (which can be drawn by enabling \"Show symmetry\") are formed from the following linear combinations:" }
                    </p>
                    <ul>
                        { for kind.rotations().iter().map(|lc| html! {
                            <li><RawSpan inner_html = utils::fmt_orbital_name_html(
                                lc.expression()
                            ) /></li>
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
                { format!(
                    "Visualized using {} points.",
                    utils::fmt_thousands_separated(state.quality() as usize)
                ) }
                </p>
            </div>
        }
    }
}

pub(crate) type InfoPanel = WithDispatch<InfoPanelImpl>;
