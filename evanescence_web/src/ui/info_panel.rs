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

    #[allow(clippy::too_many_lines)] // UIs are lengthy.
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
                let qn = state.qn();
                let num_radial_nodes = Real::num_radial_nodes(qn);
                let num_angular_nodes = Real::num_angular_nodes(qn);
                let subshell_name =
                    orbital::atomic::subshell_name(qn.l()).expect("failed to get subshell name");
                html! {
                    <p>
                        { "Viewing orbital " }
                        <RawSpan
                            inner_html = utils::fmt_orbital_name_html(Real::name(qn))
                        />
                        // Show quantum numbers here in Real (Simple) mode, since it's not shown
                        // in the picker.
                        { if state.mode() == Mode::RealSimple { html! {
                            <RawSpan
                                inner_html = format!(
                                    " (<i>n</i> = {}, <i>ℓ</i> = {}, <i>m</i> = {})",
                                    qn.n(),
                                    qn.l(),
                                    utils::fmt_replace_minus(qn.m()),
                                )
                            />
                        }} else { html!() }}
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
                    <RawSpan inner_html = utils::fmt_orbital_name_html(kind.mixture_name()) />
                };
                html! {
                    <>
                    <p>
                        {"Viewing " }
                        { kind_name.clone() }
                        { "-hybridized orbital formed by the linear combination " }
                        <RawSpan inner_html = utils::fmt_orbital_name_html(
                            kind.archetype().expression()
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
                        { for kind.iter().skip(1).map(|lc| html! {
                            <li>
                                <RawSpan
                                    inner_html = utils::fmt_orbital_name_html(lc.expression())
                                />
                            </li>
                        }) }
                    </ul>
                    </>
                }
            }
            Mode::Mo => {
                html! {
                    <p>
                        { "Viewing the " }
                        { &state.lcao().name }
                        { " molecular orbital of the "}
                        <RawSpan inner_html = "H<sub>2</sub><sup>+</sup>" />
                        { " molecule-ion with an interatomic separation of " }
                        { format!("{:.1}", state.separation()) }
                        { " Bohr radii." }
                    </p>
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
