use evanescence_core::monte_carlo::{MonteCarlo, Quality};
use evanescence_core::orbital::{self, Orbital, Qn};
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_state::{SharedState, SharedStateComponent};
use yewtil::NeqAssign;

use crate::evanescence_bridge::{plot_angular_nodes, plot_radial_nodes, plot_scatter3d_real};
use crate::plotly::config::{Config, ModeBarButtons};
use crate::plotly::isosurface::Isosurface;
use crate::plotly::layout::{Axis, Layout, LayoutRangeUpdate, Scene};
use crate::plotly::Plotly;
use crate::StateHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Traces {
    Pointillist,
    RadialNodes,
    AngularNodes,
}

pub(crate) struct PointillistVisualizationImpl {
    props: VisualizationProps,
    rendered_traces: Vec<Traces>,
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
        let trace =
            plot_scatter3d_real(orbital::Real::monte_carlo_simulate(state.qn, state.quality));

        // First render. Set up the plot and add the point cloud.
        if !self.rendered_traces.contains(&Traces::Pointillist) {
            assert!(self.rendered_traces.is_empty());

            // Manually set the plot range.
            let extent = orbital::Real::estimate_radius(state.qn);
            let axis = Axis {
                range: Some((-extent, extent)),
                ..Default::default()
            };

            Plotly::react(
                &self.props.id,
                &[trace],
                Layout {
                    ui_revision: &self.props.id,
                    scene: Some(Scene {
                        x_axis: axis,
                        y_axis: axis,
                        z_axis: axis,
                        ..Default::default()
                    }),
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
            self.rendered_traces.push(Traces::Pointillist);
        } else {
            assert!(self.rendered_traces[0] == Traces::Pointillist);
            // On subsequent renders, only update the trace.
            Plotly::delete_trace(&self.props.id, 0);
            // Relayout to set new plot range. Note that we relayout when there are no points
            // displayed to improve performance.
            Plotly::relayout(
                &self.props.id,
                LayoutRangeUpdate::new(orbital::Real::estimate_radius(state.qn)),
            );
            Plotly::add_trace_at(&self.props.id, trace, 0);
        }
    }

    fn render_or_remove_nodes(&mut self, kind: &Traces) {
        let state = self.props.handle.state();
        let current_render_status = self.rendered_traces.iter().position(|t| t == kind);

        // Test if a trace of `kind` is already present.
        if let Some(index) = current_render_status {
            // If so, we always remove it.
            Plotly::delete_trace(&self.props.id, index as _);
            self.rendered_traces.remove(index);
        }
        // There should be at most one trace of a certain kind. Since we just removed up to one,
        // there should be none left.
        assert!(!self.rendered_traces.contains(kind));

        // Check the current state to see if we should render a new trace.
        let (should_render, renderer): (bool, fn(Qn, Quality) -> Isosurface<'static>) = match kind {
            Traces::RadialNodes => (state.radial_nodes_visibility, plot_radial_nodes),
            Traces::AngularNodes => (state.angular_nodes_visibility, plot_angular_nodes),
            _ => unreachable!(),
        };

        // If so, compute and render one.
        if should_render {
            Plotly::add_trace(&self.props.id, renderer(state.qn, state.quality));
            self.rendered_traces.push(*kind);
        }
    }

    fn render_or_remove_all_nodes(&mut self) {
        self.render_or_remove_nodes(&Traces::RadialNodes);
        self.render_or_remove_nodes(&Traces::AngularNodes);
    }
}

impl Component for PointillistVisualizationImpl {
    type Message = ();
    type Properties = VisualizationProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            props,
            rendered_traces: Vec::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let diff = self.props.handle.state().diff(props.handle.state());
        self.props.neq_assign(props);
        if diff.qn_or_quality {
            self.render_pointillist();
            self.render_or_remove_all_nodes();
        }
        if diff.radial_nodes {
            self.render_or_remove_nodes(&Traces::RadialNodes);
        }
        if diff.angular_nodes {
            self.render_or_remove_nodes(&Traces::AngularNodes);
        }
        false
    }

    fn rendered(&mut self, _first_render: bool) {
        self.render_pointillist();
        self.render_or_remove_all_nodes();
    }

    fn view(&self) -> Html {
        html! {
            <div class="visualization" id = self.props.id />
        }
    }
}

pub(crate) type PointillistVisualization = SharedStateComponent<PointillistVisualizationImpl>;
