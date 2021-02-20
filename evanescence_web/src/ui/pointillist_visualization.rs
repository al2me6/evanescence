use evanescence_core::orbital::Real as RealOrbital;
use strum::{EnumCount, EnumIter, IntoEnumIterator};
use wasm_bindgen::JsValue;
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew_state::SharedStateComponent;
use yewtil::NeqAssign;

use crate::plotly::config::ModeBarButtons;
use crate::plotly::layout::{Axis, LayoutRangeUpdate, Scene};
use crate::plotly::{Config, Layout, Plotly};
use crate::plotters::pointillist as plot;
use crate::state::{Mode, State, StateHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount)]
enum Trace {
    Pointillist,
    RadialNodes,
    AngularNodes,
    CrossSectionIndicator,
}

enum TraceRenderer {
    Single(fn(&State) -> JsValue),
    Multiple(fn(&State) -> Vec<JsValue>),
}

impl Trace {
    fn should_render(self, state: &State) -> (bool, TraceRenderer) {
        let mode = state.mode();
        match self {
            Self::Pointillist => (
                true,
                TraceRenderer::Single(match mode {
                    Mode::RealSimple | Mode::Real | Mode::Hybrid => plot::real,
                    Mode::Complex => plot::complex,
                }),
            ),
            Self::RadialNodes => (
                state.mode().is_real_or_simple()
                    && state.nodes_rad()
                    && RealOrbital::num_radial_nodes(state.qn()) > 0,
                TraceRenderer::Single(plot::radial_nodes),
            ),
            Self::AngularNodes => (
                state.mode().is_real_or_simple()
                    && state.nodes_ang()
                    && RealOrbital::num_angular_nodes(state.qn()) > 0,
                TraceRenderer::Single(plot::angular_nodes),
            ),
            Self::CrossSectionIndicator => (
                (state.mode().is_real_or_simple() || state.mode().is_hybrid())
                    && state.supplement().is_cross_section(),
                TraceRenderer::Single(plot::cross_section_indicator),
            ),
        }
    }
}

pub(crate) struct PointillistVisualizationImpl {
    handle: StateHandle,
    rendered_traces: Vec<Trace>,
}

impl PointillistVisualizationImpl {
    const ID: &'static str = "pointillist";

    fn rerender_all(&mut self) {
        let state = self.handle.state();

        // Clear all old traces.
        Plotly::delete_traces(
            Self::ID,
            (0..self.rendered_traces.len())
                .map(|i| JsValue::from_f64(i as _))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
        self.rendered_traces.clear();

        // Relayout to set new plot range. Note that we relayout when there are no points
        // displayed to improve performance.
        Plotly::relayout(
            Self::ID,
            LayoutRangeUpdate::new(state.estimate_radius()).into(),
        );

        // And compute new ones.
        let rendered_traces = &mut self.rendered_traces;
        let mut traces_to_render = Vec::with_capacity(rendered_traces.capacity());
        for kind in Trace::iter() {
            let (should_render, renderer) = kind.should_render(state);
            if should_render {
                match renderer {
                    TraceRenderer::Single(renderer) => {
                        rendered_traces.push(kind);
                        traces_to_render.push(renderer(state));
                    }
                    TraceRenderer::Multiple(renderer) => {
                        let traces = renderer(state);
                        rendered_traces.extend(itertools::repeat_n(kind, traces.len()));
                        traces_to_render.extend(traces.into_iter());
                    }
                }
            }
        }
        Plotly::add_traces(Self::ID, traces_to_render.into_boxed_slice());
    }

    fn add_or_remove_trace(&mut self, kind: Trace) {
        // This function should not be touching the pointillist trace, since if that needs to be
        // changed then all other traces must also change.
        assert!(kind != Trace::Pointillist);

        let state = self.handle.state();

        // Test if traces of this kind are already rendered.
        if self.rendered_traces.contains(&kind) {
            // If so, remove all traces of this kind from the plot.
            Plotly::delete_traces(
                Self::ID,
                self.rendered_traces // Get all indices of matching traces.
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, &t)| (t == kind).then(|| idx as f64))
                    .map(JsValue::from_f64)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            );
            // And also remove them from the record.
            let _ = self.rendered_traces.drain_filter(|&mut t| t == kind);
        }
        // There should be no traces of this kind left.
        assert!(!self.rendered_traces.contains(&kind));

        // Check the current state to see if we should render new traces.
        let (should_render, renderer) = kind.should_render(state);

        // If so, compute and render them.
        if should_render {
            match renderer {
                TraceRenderer::Single(renderer) => {
                    Plotly::add_trace(Self::ID, renderer(state));
                    self.rendered_traces.push(kind);
                }
                TraceRenderer::Multiple(renderer) => {
                    let traces = renderer(state);
                    self.rendered_traces
                        .extend(itertools::repeat_n(kind, traces.len()));
                    Plotly::add_traces(Self::ID, traces.into_boxed_slice());
                }
            }
        }
    }
}

impl Component for PointillistVisualizationImpl {
    type Message = ();
    type Properties = StateHandle;

    fn create(handle: StateHandle, _link: ComponentLink<Self>) -> Self {
        Self {
            handle,
            rendered_traces: Vec::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, handle: StateHandle) -> ShouldRender {
        let old_state = self.handle.state();
        let new_state = handle.state();

        let all = new_state.is_new_orbital(old_state) || new_state.quality() != old_state.quality();
        let nodes_ang = new_state.nodes_ang() != old_state.nodes_ang();
        let nodes_rad = new_state.nodes_rad() != old_state.nodes_rad();
        let cross_section = (old_state.supplement().is_cross_section()
            || new_state.supplement().is_cross_section())
            && old_state.supplement() != new_state.supplement();

        self.handle.neq_assign(handle);
        if all {
            self.rerender_all();
        } else {
            [
                (nodes_rad, Trace::RadialNodes),
                (nodes_ang, Trace::AngularNodes),
                (cross_section, Trace::CrossSectionIndicator),
            ]
            .iter()
            .filter(|(should_render, _)| *should_render)
            .for_each(|(_, kind)| self.add_or_remove_trace(*kind));
        }
        false
    }

    fn rendered(&mut self, first_render: bool) {
        assert!(first_render);
        assert!(self.rendered_traces.is_empty());

        let state = self.handle.state();

        assert!(state.mode().is_real_or_simple());

        // Manually set the plot range to prevent jumping.
        let axis = Axis::from_range_of(state.qn());
        Plotly::react(
            Self::ID,
            vec![plot::real(state)].into_boxed_slice(),
            Layout {
                drag_mode_str: Some("orbit"),
                ui_revision: Some("pointillist"),
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
            <div class="visualization" id = Self::ID />
        }
    }
}

pub(crate) type PointillistVisualization = SharedStateComponent<PointillistVisualizationImpl>;
