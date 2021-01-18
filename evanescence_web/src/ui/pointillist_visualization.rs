use evanescence_core::monte_carlo::MonteCarlo;
use evanescence_core::orbital::{self, Orbital};
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_state::{SharedState, SharedStateComponent};
use yewtil::NeqAssign;

use crate::evanescence_bridge;
use crate::plotly::config::{Config, ModeBarButtons};
use crate::plotly::layout::{Layout, Scene};
use crate::plotly::Plotly;
use crate::StateHandle;

pub(crate) struct PointillistVisualizationImpl {
    props: VisualizationProps,
    has_rendered_pointillist: bool,
    has_rendered_nodes: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub(crate) struct VisualizationProps {
    pub(crate) id: String,
    #[prop_or_default]
    pub(crate) handle: StateHandle,
}

impl SharedState for VisualizationProps {
    type Handle = StateHandle;

    fn handle(&mut self) -> &mut Self::Handle {
        self.handle.handle()
    }
}

impl PointillistVisualizationImpl {
    fn render_pointillist(&mut self) {
        let state = self.props.handle.state();
        let trace = evanescence_bridge::into_scatter3d_real(orbital::Real::monte_carlo_simulate(
            state.qn,
            state.quality,
        ));

        if !self.has_rendered_pointillist {
            Plotly::react(
                &self.props.id,
                &[trace],
                Layout {
                    ui_revision: &self.props.id,
                    scene: Some(Scene::default()),
                    ..Default::default()
                },
                Config {
                    mode_bar_buttons_to_remove: &[
                        ModeBarButtons::ResetCameraLastSave3d,
                        ModeBarButtons::HoverClosest3d,
                    ],
                    ..Default::default()
                },
            );
            self.has_rendered_pointillist = true;
        } else {
            Plotly::delete_trace(&self.props.id, 0);
            Plotly::add_trace_at(&self.props.id, trace, 0);
        }
    }

    fn render_or_remove_nodes(&mut self) {
        let state = self.props.handle.state();
        if self.has_rendered_nodes {
            Plotly::delete_trace(&self.props.id, 1);
        }
        if state.show_nodes {
            let trace = evanescence_bridge::into_nodal_surface(orbital::Real::sample_region(
                state.qn,
                state.quality.for_isosurface(),
            ));
            Plotly::add_trace(&self.props.id, trace);
            self.has_rendered_nodes = true;
        } else {
            self.has_rendered_nodes = false;
        }
    }
}

impl Component for PointillistVisualizationImpl {
    type Message = ();
    type Properties = VisualizationProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            props,
            has_rendered_pointillist: false,
            has_rendered_nodes: false,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let (render_all, render_nodes_only) = {
            let state = self.props.handle.state();
            let new_state = props.handle.state();
            (
                !(state.qn == new_state.qn && state.quality == new_state.quality),
                state.show_nodes != new_state.show_nodes,
            )
        };
        self.props.neq_assign(props);
        if render_all {
            self.render_pointillist();
            self.render_or_remove_nodes();
        }
        if render_nodes_only {
            self.render_or_remove_nodes();
        }
        false
    }

    fn rendered(&mut self, _first_render: bool) {
        self.render_pointillist();
        self.render_or_remove_nodes();
    }

    fn view(&self) -> Html {
        html! {
            <div class="visualization" id = self.props.id />
        }
    }
}

pub(crate) type PointillistVisualization = SharedStateComponent<PointillistVisualizationImpl>;
