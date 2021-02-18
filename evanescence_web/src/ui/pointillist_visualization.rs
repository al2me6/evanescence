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

impl Trace {
    fn should_render(self, state: &State) -> (bool, fn(&State) -> JsValue) {
        let mode = state.mode();
        match self {
            Self::Pointillist => (
                true,
                match mode {
                    Mode::RealSimple | Mode::Real | Mode::Hybridized => plot::real,
                    Mode::Complex => plot::complex,
                },
            ),
            Self::RadialNodes => (
                state.mode().is_real_or_simple()
                    && state.nodes_rad()
                    // Note that this depends on the short-circuiting behavior of `&&`!
                    && RealOrbital::num_radial_nodes(state.qn()) > 0,
                plot::radial_nodes,
            ),
            Self::AngularNodes => (
                state.mode().is_real_or_simple()
                    && state.nodes_ang()
                    // Note that this depends on the short-circuiting behavior of `&&`!
                    && RealOrbital::num_angular_nodes(state.qn()) > 0,
                plot::angular_nodes,
            ),
            Self::CrossSectionIndicator => (
                (state.mode().is_real_or_simple() || state.mode().is_hybridized())
                    && state.supplement().is_cross_section(),
                plot::cross_section_indicator,
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
        let mut traces = Vec::with_capacity(Trace::COUNT);
        for t in Trace::iter() {
            let (should_render, renderer) = t.should_render(state);
            if should_render {
                rendered_traces.push(t);
                traces.push(renderer(state));
            }
        }
        Plotly::add_traces(Self::ID, traces.into_boxed_slice());
    }

    fn add_or_remove_trace(&mut self, kind: Trace) {
        let state = self.handle.state();

        // Test if a trace of `kind` is already present.
        if let Some(index) = self.rendered_traces.iter().position(|&t| t == kind) {
            // If so, we always remove it.
            Plotly::delete_trace(Self::ID, index as _);
            self.rendered_traces.remove(index);
        }
        // There should be at most one trace of a certain kind. Since we just removed up to one,
        // there should be none left.
        assert!(!self.rendered_traces.contains(&kind));

        // Check the current state to see if we should render a new trace.
        let (should_render, renderer) = kind.should_render(state);

        // If so, compute and render one.
        if should_render {
            Plotly::add_trace(Self::ID, renderer(state));
            self.rendered_traces.push(kind);
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

        assert!(state.mode() == Mode::RealSimple);

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
