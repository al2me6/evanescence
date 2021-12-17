use evanescence_core::orbital::molecular::Molecular;
use evanescence_core::orbital::{self, Complex, Orbital, Real1};
use evanescence_web::components::raw::RawSpan;
use evanescence_web::state::{AppDispatch, Mode};
use evanescence_web::utils;
use yew::prelude::*;
use yewdux::prelude::*;

pub struct InfoPanelImpl {}

impl Component for InfoPanelImpl {
    type Message = ();
    type Properties = AppDispatch;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    #[allow(clippy::too_many_lines)] // UIs are lengthy.
    fn view(&self, ctx: &Context<Self>) -> Html {
        fn node_pluralize(n: u32) -> &'static str {
            if n == 1 {
                "node"
            } else {
                "nodes"
            }
        }

        let state = ctx.props().state();

        let description = match state.mode() {
            Mode::RealSimple | Mode::Real => {
                let qn = state.qn();
                let subshell_name = orbital::atomic::subshell_name(qn.l());

                let num_radial_nodes = Real1::num_radial_nodes(qn);
                let num_angular_nodes = Real1::num_angular_nodes(qn);
                let nodes_description = format!(
                    " {num_radial_nodes} radial {} and {num_angular_nodes} angular{} {}.",
                    node_pluralize(num_radial_nodes),
                    match (
                        Real1::num_conical_nodes(&qn.into()),
                        Real1::num_planar_nodes(&qn.into()),
                    ) {
                        // English is hard.
                        (0, 0) => "".to_owned(),
                        (1, 0) => " (conical)".to_owned(),
                        (0, 1) => " (planar)".to_owned(),
                        (2, 0) => " (both conical)".to_owned(),
                        (0, 2) => " (both planar)".to_owned(),
                        (c, 0) if c > 0 => " (all conical)".to_owned(),
                        (0, p) if p > 0 => " (all planar)".to_owned(),
                        (c, p) => format!(" ({c} conical, {p} planar)"),
                    },
                    node_pluralize(num_angular_nodes),
                );

                html! {
                    <p>
                        { "Viewing orbital " }
                        <RawSpan
                            inner_html = { utils::fmt_orbital_name_html(Real1::name(qn)) }
                        />
                        // Show quantum numbers here in Real (Simple) mode, since it's not shown
                        // in the picker.
                        if state.mode() == Mode::RealSimple {
                            <RawSpan
                                inner_html = { format!(
                                    " (<i>n</i> = {}, <i>ℓ</i> = {}, <i>m</i> = {})",
                                    qn.n(),
                                    qn.l(),
                                    utils::fmt_replace_minus(qn.m()),
                                ) }
                            />
                        }
                        { ", which " }
                        if let Some(subshell) = subshell_name {
                            { "is " }
                            { if "sfhi".contains(subshell) { "an " } else { "a " } } // English is hard.
                            <i>{ subshell }</i>
                            { " orbital with" }
                        } else {
                            { "has" }
                        }
                        { nodes_description }
                    </p>
                }
            }
            Mode::Complex => html! {
                <p>
                    { "Viewing orbital " }
                    <RawSpan inner_html = { Complex::name(state.qn()) } />
                    { "." }
                </p>
            },
            Mode::Hybrid => {
                let kind = state.hybrid_kind();
                let kind_name = html! {
                    <RawSpan inner_html = { utils::fmt_orbital_name_html(kind.mixture_name()) } />
                };
                html! {
                    <>
                    <p>
                        { "Viewing one of " }
                        { kind.count() }
                        { " " }
                        { kind_name.clone() }
                        { "-hybridized orbitals, which have " }
                        { kind.symmetry() }
                        { " symmetry. They are formed from the following linear combinations:" }
                    </p>
                    <ul>
                        { for kind.iter().map(|lc| html! {
                            <li>
                                <RawSpan
                                    inner_html = { utils::fmt_orbital_name_html(lc.expression()) }
                                />
                                { match (lc == kind.archetype(), state.silhouettes()) {
                                    (true, true) => " (displayed & outlined)",
                                    (true, false) => " (displayed)",
                                    (false, true) => " (outlined)",
                                    (false, false) => "",
                                }}
                            </li>
                        }) }
                    </ul>
                    <p>
                        { "To draw all " }
                        { kind_name }
                        { r#" orbitals to visualize their symmetry, enable the "Show symmetry" toggle."# }
                    </p>
                    </>
                }
            }
            Mode::Mo => {
                html! {
                    <p>
                        { "Viewing a " }
                        <RawSpan inner_html = { Molecular::orbital_type(&state.lcao()) } />
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

pub type InfoPanel = WithDispatch<InfoPanelImpl>;
