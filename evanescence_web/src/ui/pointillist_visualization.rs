use evanescence_core::orbital::Real as RealOrbital;
use strum::{EnumIter, IntoEnumIterator};
use wasm_bindgen::JsValue;
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew_state::SharedStateComponent;
use yewtil::NeqAssign;

use crate::plotly::config::ModeBarButtons;
use crate::plotly::layout::{LayoutRangeUpdate, Scene};
use crate::plotly::{Config, Layout, Plotly};
use crate::plotters::pointillist as plot;
use crate::state::{Mode, State, StateHandle};

enum TraceRenderer {
    Single(fn(&State) -> JsValue),
    Multiple(fn(&State) -> Vec<JsValue>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
enum Trace {
    Pointillist,
    RadialNodes,
    AngularNodes,
    CrossSectionIndicator,
    Silhouette,
}

impl Trace {
    fn should_render(self, state: &State) -> bool {
        match self {
            Self::Pointillist => true,
            Self::RadialNodes => {
                state.mode().is_real_or_simple()
                    && state.nodes_rad()
                    && RealOrbital::num_radial_nodes(state.qn()) > 0
            }
            Self::AngularNodes => {
                state.mode().is_real_or_simple()
                    && state.nodes_ang()
                    && RealOrbital::num_angular_nodes(state.qn()) > 0
            }
            Self::CrossSectionIndicator => state.supplement().is_cross_section(),
            Self::Silhouette => state.mode().is_hybrid() && state.silhouettes(),
        }
    }

    fn renderer(self, state: &State) -> TraceRenderer {
        use TraceRenderer::{Multiple, Single};
        match self {
            Self::Pointillist => match state.mode() {
                Mode::RealSimple | Mode::Real | Mode::Hybrid => Single(plot::real),
                Mode::Complex => Single(plot::complex),
            },
            Self::RadialNodes => Single(plot::radial_nodes),
            Self::AngularNodes => Single(plot::angular_nodes),
            Self::CrossSectionIndicator => Single(plot::cross_section_indicator),
            Self::Silhouette => Multiple(plot::silhouettes),
        }
    }

    fn render_to_vec(self, state: &State) -> Vec<JsValue> {
        match self.renderer(state) {
            TraceRenderer::Single(renderer) => vec![renderer(state)],
            TraceRenderer::Multiple(renderer) => renderer(state),
        }
    }
}

pub(crate) struct PointillistVisualizationImpl {
    handle: StateHandle,
    rendered_kinds: Vec<Trace>,
}

impl PointillistVisualizationImpl {
    const ID: &'static str = "pointillist";

    fn init_plot(&mut self) {
        assert!(self.rendered_kinds.is_empty());

        // Initialize empty plot.
        Plotly::react(
            Self::ID,
            vec![].into_boxed_slice(),
            Layout {
                drag_mode_str: Some("orbit"),
                ui_revision: Some("pointillist"),
                scene: Some(Scene::default()),
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

        self.rerender_all();
    }

    fn rerender_all(&mut self) {
        let state = self.handle.state();

        // Clear all old traces.
        Plotly::delete_traces(
            Self::ID,
            (0..self.rendered_kinds.len())
                .map(|i| JsValue::from_f64(i as _))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
        self.rendered_kinds.clear();

        // Relayout to set new plot range. Note that we relayout when there are no points
        // displayed to improve performance.
        Plotly::relayout(
            Self::ID,
            LayoutRangeUpdate::new(state.estimate_radius()).into(),
        );

        // And compute new ones.
        let rendered_kinds = &mut self.rendered_kinds;
        let mut traces_to_render = Vec::with_capacity(rendered_kinds.capacity());
        Trace::iter()
            .filter(|kind| kind.should_render(state))
            .for_each(|kind| {
                log::debug!("Rerendering {:?}.", kind);
                let traces = kind.render_to_vec(state);
                rendered_kinds.extend(itertools::repeat_n(kind, traces.len()));
                traces_to_render.extend(traces.into_iter());
            });
        Plotly::add_traces(Self::ID, traces_to_render.into_boxed_slice());
    }

    fn add_or_remove_kind(&mut self, kind: Trace) {
        // This function should not be touching the pointillist trace, since if that needs to be
        // changed then all other traces must also change.
        assert_ne!(kind, Trace::Pointillist);

        let state = self.handle.state();

        // Test if traces of this kind are already rendered.
        if self.rendered_kinds.contains(&kind) {
            // If so, remove all traces of this kind from the plot.
            Plotly::delete_traces(
                Self::ID,
                self.rendered_kinds // Get all indices of matching traces.
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, &t)| (t == kind).then(|| idx as f64))
                    .map(JsValue::from_f64)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            );
            // And also remove them from the record.
            let _removed = self.rendered_kinds.drain_filter(|&mut t| t == kind);

            #[allow(clippy::used_underscore_binding)] // This is conditionally used.
            #[cfg(debug_assertions)]
            _removed.for_each(|kind| log::debug!("Removing {:?}.", kind));
        }
        // There should be no traces of this kind left.
        assert!(!self.rendered_kinds.contains(&kind));

        // Check the current state to see if we should render new traces. If so, compute and
        // render them.
        if kind.should_render(state) {
            log::debug!("Adding {:?}.", kind);
            let traces = kind.render_to_vec(state);
            self.rendered_kinds
                .extend(itertools::repeat_n(kind, traces.len()));
            Plotly::add_traces(Self::ID, traces.into_boxed_slice());
        }
    }
}

impl Component for PointillistVisualizationImpl {
    type Message = ();
    type Properties = StateHandle;

    fn create(handle: StateHandle, _link: ComponentLink<Self>) -> Self {
        Self {
            handle,
            rendered_kinds: Vec::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, handle: StateHandle) -> ShouldRender {
        #[derive(Clone, Copy)]
        enum RenderDirective {
            All,
            Single(Trace),
            Skip,
        }

        let old = self.handle.state();
        let new = handle.state();

        let directive = if new.is_new_orbital(old) || new.quality() != old.quality() {
            RenderDirective::All
        } else {
            let change = [
                (new.nodes_rad() != old.nodes_rad(), Trace::RadialNodes),
                (new.nodes_ang() != old.nodes_ang(), Trace::AngularNodes),
                (
                    (old.supplement().is_cross_section() || new.supplement().is_cross_section())
                        && old.supplement() != new.supplement(),
                    Trace::CrossSectionIndicator,
                ),
                (old.silhouettes() != new.silhouettes(), Trace::Silhouette),
            ]
            .iter()
            .filter_map(|&(changed, kind)| changed.then(|| kind))
            .collect::<Vec<_>>();

            assert!(change.len() <= 1, "only one trace can be changed at once");

            change
                .get(0)
                .map_or(RenderDirective::Skip, |&kind| RenderDirective::Single(kind))
        };

        // Note that the rendering operation requires the state to be updated!
        self.handle.neq_assign(handle);

        match directive {
            RenderDirective::All => self.rerender_all(),
            RenderDirective::Single(kind) => self.add_or_remove_kind(kind),
            RenderDirective::Skip => {}
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="visualization" id = Self::ID />
        }
    }

    fn rendered(&mut self, first_render: bool) {
        assert!(first_render);
        self.init_plot();
    }
}

pub(crate) type PointillistVisualization = SharedStateComponent<PointillistVisualizationImpl>;
