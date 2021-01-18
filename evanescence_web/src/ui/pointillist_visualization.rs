use evanescence_core::monte_carlo::MonteCarlo;
use evanescence_core::orbital::{self, wavefunctions, Orbital};
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_state::{SharedState, SharedStateComponent};
use yewtil::NeqAssign;

use crate::evanescence_bridge::{plot_isosurface, plot_scatter3d_real};
use crate::plotly::config::{Config, ModeBarButtons};
use crate::plotly::layout::{Axis, Layout, LayoutRangeUpdate, Scene};
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
        let trace =
            plot_scatter3d_real(orbital::Real::monte_carlo_simulate(state.qn, state.quality));

        // First render, set up the plot too.
        if !self.has_rendered_pointillist {
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
            self.has_rendered_pointillist = true;
        } else {
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

    fn render_or_remove_nodes(&mut self) {
        let state = self.props.handle.state();
        // We always need to remove the old isosurface.
        if self.has_rendered_nodes {
            Plotly::delete_trace(&self.props.id, -1);
            Plotly::delete_trace(&self.props.id, -1);
        }
        // Add a new one if necessary.
        if state.nodes_visibility {
            let radial_trace = plot_isosurface(
                orbital::sample_region_for::<wavefunctions::Radial>(
                    state.qn,
                    state.quality.for_isosurface(),
                ),
                false,
            );
            let angular_trace = plot_isosurface(
                orbital::sample_region_for::<wavefunctions::RealSphericalHarmonic>(
                    state.qn,
                    state.quality.for_isosurface(),
                ),
                state.qn.l() > 6 && state.qn.m().abs() > 5,
            );
            Plotly::add_trace(&self.props.id, radial_trace);
            Plotly::add_trace(&self.props.id, angular_trace);
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
        let diff = self.props.handle.state().diff(props.handle.state());
        self.props.neq_assign(props);

        if diff.qn_or_quality {
            self.render_pointillist();
            self.render_or_remove_nodes();
        }
        if diff.nodes_visibility {
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
