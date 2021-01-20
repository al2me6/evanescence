use evanescence_core::monte_carlo::Quality;
use evanescence_core::orbital::{self, Orbital, Qn};
use strum::{EnumIter, IntoEnumIterator};
use wasm_bindgen::JsValue;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_state::{SharedState, SharedStateComponent};
use yewtil::NeqAssign;

use crate::evanescence_bridge::{plot_angular_nodes, plot_pointillist_real, plot_radial_nodes};
use crate::plotly::config::ModeBarButtons;
use crate::plotly::layout::{Axis, LayoutRangeUpdate, Scene};
use crate::plotly::{Config, Layout, Plotly};
use crate::{State, StateHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
enum Trace {
    Pointillist,
    RadialNodes,
    AngularNodes,
}

impl Trace {
    fn should_render(self, state: &State) -> (bool, fn(Qn, Quality) -> JsValue) {
        match self {
            Self::Pointillist => (true, plot_pointillist_real),
            Self::RadialNodes => (state.nodes_show_radial, plot_radial_nodes),
            Self::AngularNodes => (state.nodes_show_angular, plot_angular_nodes),
        }
    }
}

pub(crate) struct PointillistVisualizationImpl {
    props: VisualizationProps,
    rendered_traces: Vec<Trace>,
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
    fn rerender_all(&self) {
        let state = self.props.handle.state();

        // Validate that the currently rendered traces match what should be rendered according
        // to the state.
        Trace::iter().for_each(|t| {
            let (expected_render_state, _) = t.should_render(state);
            assert!(!(self.rendered_traces.contains(&t) ^ expected_render_state));
        });

        // Clear all old traces.
        Plotly::delete_traces(
            &self.props.id,
            (0..self.rendered_traces.len())
                .into_iter()
                .map(|i| JsValue::from_f64(i as _))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );

        // Relayout to set new plot range. Note that we relayout when there are no points
        // displayed to improve performance.
        Plotly::relayout(
            &self.props.id,
            LayoutRangeUpdate::new(orbital::Real::estimate_radius(state.qn)).into(),
        );

        // And compute new ones in the same order.
        let traces: Vec<JsValue> = self
            .rendered_traces
            .iter()
            .map(|&t| t.should_render(state))
            .map(|(_, renderer)| renderer(state.qn, state.quality))
            .collect();
        Plotly::add_traces(&self.props.id, traces.into_boxed_slice());
    }

    fn add_or_remove_trace(&mut self, kind: Trace) {
        let state = self.props.handle.state();

        // Test if a trace of `kind` is already present.
        if let Some(index) = self.rendered_traces.iter().position(|&t| t == kind) {
            // If so, we always remove it.
            Plotly::delete_trace(&self.props.id, index as _);
            self.rendered_traces.remove(index);
        }
        // There should be at most one trace of a certain kind. Since we just removed up to one,
        // there should be none left.
        assert!(!self.rendered_traces.contains(&kind));

        // Check the current state to see if we should render a new trace.
        let (should_render, renderer) = kind.should_render(state);

        // If so, compute and render one.
        if should_render {
            Plotly::add_trace(&self.props.id, renderer(state.qn, state.quality));
            self.rendered_traces.push(kind);
        }
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
            self.rerender_all();
        } else {
            if diff.nodes_radial {
                self.add_or_remove_trace(Trace::RadialNodes);
            }
            if diff.nodes_angular {
                self.add_or_remove_trace(Trace::AngularNodes);
            }
        }
        false
    }

    fn rendered(&mut self, first_render: bool) {
        assert!(first_render);
        assert!(self.rendered_traces.is_empty());

        let state = self.props.handle.state();

        // Manually set the plot range.
        let extent = orbital::Real::estimate_radius(state.qn);
        let axis = Axis {
            range: Some((-extent, extent)),
            ..Default::default()
        };

        Plotly::react(
            &self.props.id,
            vec![plot_pointillist_real(state.qn, state.quality)].into_boxed_slice(),
            Layout {
                ui_revision: true,
                scene: Some(Scene {
                    x_axis: axis,
                    y_axis: axis,
                    z_axis: axis,
                    ..Default::default()
                }),
                ..Default::default()
            }
            .into(),
            Config {
                mode_bar_buttons_to_remove: &[
                    ModeBarButtons::ResetCameraLastSave3d,
                    ModeBarButtons::HoverClosest3d,
                ],
                ..Default::default()
            }
            .into(),
        );
        self.rendered_traces.push(Trace::Pointillist);
    }

    fn view(&self) -> Html {
        html! {
            <div class="visualization" id = self.props.id />
        }
    }
}

pub(crate) type PointillistVisualization = SharedStateComponent<PointillistVisualizationImpl>;
