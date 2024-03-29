use std::mem;
use std::rc::Rc;

use evanescence_core::orbital::AtomicReal;
use evanescence_web::plotly::config::ModeBarButtons;
use evanescence_web::plotly::layout::{LayoutRangeUpdate, Scene};
use evanescence_web::plotly::{Config, Layout, Plotly};
use evanescence_web::plotters::pointillist as plot;
use evanescence_web::state::{Mode, State, StateDispatch};
use evanescence_web::time_scope;
use evanescence_web::utils::{b16_colors, StopwatchSlot};
use strum::{EnumIter, IntoEnumIterator};
use wasm_bindgen::JsValue;
use yew::prelude::*;

pub static POINTILLIST_STOPWATCH: StopwatchSlot = StopwatchSlot("pointillist-stopwatch");

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
enum Trace {
    Pointillist,
    NodesRadial,
    NodesAngular,
    CrossSectionIndicator,
    Silhouettes,
    Nodes,
    // NucleusMarkers,
}

impl Trace {
    fn should_render(self, state: &State) -> bool {
        match self {
            Self::Pointillist => true,
            Self::NodesRadial => {
                state.mode().is_real_or_simple()
                    && state.nodes_rad()
                    && AtomicReal::num_radial_nodes(*state.qn()) > 0
            }
            Self::NodesAngular => {
                state.mode().is_real_or_simple()
                    && state.nodes_ang()
                    && AtomicReal::num_angular_nodes(*state.qn()) > 0
            }
            Self::CrossSectionIndicator => state.supplement().is_cross_section(),
            Self::Silhouettes => state.mode().is_hybrid() && state.silhouettes(),
            Self::Nodes => state.mode().is_hybrid() && state.nodes(),
        }
    }

    fn render(self, state: &State) -> Vec<JsValue> {
        let renderer = match self {
            Self::Pointillist => match state.mode() {
                Mode::RealSimple | Mode::RealFull | Mode::Hybrid => plot::real,
                Mode::Complex => plot::complex,
            },
            Self::NodesRadial => plot::nodes_radial,
            Self::NodesAngular => plot::nodes_angular,
            Self::CrossSectionIndicator => plot::cross_section_indicator,
            Self::Silhouettes => plot::silhouettes,
            Self::Nodes => plot::nodes_combined,
        };
        renderer(state)
    }
}

pub struct PointillistVisualization {
    current_state: Rc<State>,
    rendered_kinds: Vec<Trace>,
}

impl PointillistVisualization {
    const ID: &'static str = "pointillist";

    fn init_plot(&mut self) {
        assert!(self.rendered_kinds.is_empty());

        // Initialize empty plot.
        Plotly::react(
            Self::ID,
            vec![].into_boxed_slice(),
            Layout {
                drag_mode_str: Some("orbit"),
                ui_revision: Some("pointillist".to_owned()),
                scene: Some(Scene::default()),
                paper_bgcolor: b16_colors::BASE[0x01],
                plot_bgcolor: b16_colors::BASE[0x01],
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
        let state = &self.current_state;

        let _timer = time_scope!(
            in: &POINTILLIST_STOPWATCH,
            "[{}][{}] Full Pointillist render",
            state.debug_description(),
            state.quality(),
        );

        // Clear all old traces.
        self.rendered_kinds
            .iter()
            .filter(|&&kind| kind != Trace::Pointillist)
            .for_each(|kind| log::debug!("Removing {kind:?}."));
        Plotly::delete_traces(Self::ID, 0..(self.rendered_kinds.len() as isize));
        self.rendered_kinds.clear();

        // Relayout to set new plot range. Note that we relayout when there are no points
        // displayed to improve performance.
        Plotly::relayout(Self::ID, LayoutRangeUpdate::new(state.bound()).into());

        // And compute new ones.
        let rendered_kinds = &mut self.rendered_kinds;
        let mut traces_to_render = Vec::with_capacity(rendered_kinds.capacity());
        Trace::iter()
            .filter(|kind| kind.should_render(state))
            .for_each(|kind| {
                log::debug!("Rerendering {kind:?}.");
                let traces = kind.render(state);
                rendered_kinds.extend(itertools::repeat_n(kind, traces.len()));
                traces_to_render.extend(traces.into_iter());
            });
        Plotly::add_traces(Self::ID, traces_to_render.into_boxed_slice());
    }

    fn add_or_remove_kind(&mut self, kind: Trace) {
        let state = &self.current_state;

        let _timer = time_scope!(
            in: &POINTILLIST_STOPWATCH,
            "[{}][{}] Render {:?} trace",
            state.debug_description(),
            state.quality(),
            kind,
        );

        // Test if traces of this kind are already rendered.
        if self.rendered_kinds.contains(&kind) {
            // If so, remove all traces of this kind from the plot.
            Plotly::delete_traces(
                Self::ID,
                self.rendered_kinds // Get all indices of matching traces.
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, &t)| (t == kind).then_some(idx as isize)),
            );
            // And also remove them from the record.
            let removed = self.rendered_kinds.drain_filter(|&mut t| t == kind);
            removed.for_each(|kind| log::debug!("Removing {kind:?}."));
        }
        // There should be no traces of this kind left.
        assert!(!self.rendered_kinds.contains(&kind));

        // Check the current state to see if we should render new traces. If so, compute and
        // render them.
        if kind.should_render(state) {
            log::debug!("Adding {kind:?}.");
            let traces = kind.render(state);
            self.rendered_kinds
                .extend(itertools::repeat_n(kind, traces.len()));
            Plotly::add_traces(Self::ID, traces.into_boxed_slice());
        }
    }
}

impl Component for PointillistVisualization {
    type Message = ();
    type Properties = StateDispatch;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            current_state: ctx.props().state(),
            rendered_kinds: Vec::new(),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        #[derive(Clone, Copy)]
        enum RenderDirective {
            All,
            Single(Trace),
            Skip,
        }

        let old = &mem::replace(&mut self.current_state, ctx.props().state());
        let new = &self.current_state;

        let directive = if new.mode().is_real_or_simple()
            && !new.is_new_orbital(old)
            && new.quality() != old.quality()
        {
            assert_eq!(new.nodes_ang(), old.nodes_ang());
            assert_eq!(new.nodes_rad(), old.nodes_rad());
            RenderDirective::Single(Trace::Pointillist)
        } else if new.is_new_orbital(old) || new.quality() != old.quality() {
            RenderDirective::All
        } else {
            let mut possible_changes = [
                (new.nodes_rad() != old.nodes_rad(), Trace::NodesRadial),
                (new.nodes_ang() != old.nodes_ang(), Trace::NodesAngular),
                (
                    (new.supplement().is_cross_section() || old.supplement().is_cross_section())
                        && new.supplement() != old.supplement(),
                    Trace::CrossSectionIndicator,
                ),
                (new.silhouettes() != old.silhouettes(), Trace::Silhouettes),
                (new.nodes() != old.nodes(), Trace::Nodes),
            ]
            .into_iter()
            .filter_map(|(changed, kind)| changed.then_some(kind));

            let directive = possible_changes
                .next()
                // Tuple enum variant is used as a function in the second argument.
                .map_or(RenderDirective::Skip, RenderDirective::Single);
            assert!(
                possible_changes.next().is_none(),
                "at most one trace can be changed"
            );
            directive
        };

        match directive {
            RenderDirective::All => self.rerender_all(),
            RenderDirective::Single(kind) => self.add_or_remove_kind(kind),
            RenderDirective::Skip => {}
        }

        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class = "visualization" id = { Self::ID }>
                <p class = "stopwatch-slot" id = { POINTILLIST_STOPWATCH.0 } />
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        assert!(first_render);
        self.init_plot();
    }
}
