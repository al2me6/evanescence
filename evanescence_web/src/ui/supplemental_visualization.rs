use wasm_bindgen::JsValue;
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew_state::SharedStateComponent;
use yewtil::NeqAssign;

use super::descriptions::DESC;
use crate::components::raw::RawSpan;
use crate::plotly::config::ModeBarButtons;
use crate::plotly::{Config, Plotly};
use crate::plotters::{self, supplemental as plot};
use crate::state::{State, StateHandle, Visualization};
use crate::utils::{self, Timer};

pub(crate) struct SupplementalVisualizationImpl {
    handle: StateHandle,
}

impl SupplementalVisualizationImpl {
    const ID: &'static str = "supplemental";

    fn rerender(&mut self) {
        let state = self.handle.state();

        let renderer: fn(&State) -> (JsValue, JsValue) = match state.supplement() {
            Visualization::None => return, // No need to render.
            Visualization::RadialWavefunction | Visualization::RadialProbabilityDistribution => {
                plot::radial
            }
            Visualization::CrossSectionXY
            | Visualization::CrossSectionYZ
            | Visualization::CrossSectionZX => plot::cross_section,
            Visualization::Isosurface3D => plot::isosurface_3d,
        };

        log::debug!("Rerendering {:?}.", state.supplement());

        let _timer = Timer::time_current_scope(format!(
            "[{}][{}] Render {:?} supplement",
            state.debug_description(),
            state.quality(),
            state.supplement(),
        ));

        let (trace, layout) = renderer(state);
        Plotly::react(
            Self::ID,
            vec![trace].into_boxed_slice(),
            layout,
            Config {
                mode_bar_buttons_to_remove: &[
                    ModeBarButtons::AutoScale2d,
                    ModeBarButtons::HoverClosest3d,
                    ModeBarButtons::HoverCompareCartesian,
                    ModeBarButtons::Lasso2d,
                    ModeBarButtons::ResetCameraLastSave3d,
                    ModeBarButtons::Select2d,
                    ModeBarButtons::ToggleSpikelines,
                    ModeBarButtons::ZoomIn2d,
                    ModeBarButtons::ZoomOut2d,
                ],
                ..Default::default()
            }
            .into(),
        )
    }
}

impl Component for SupplementalVisualizationImpl {
    type Message = ();
    type Properties = StateHandle;

    fn create(handle: StateHandle, _link: ComponentLink<Self>) -> Self {
        Self { handle }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, handle: StateHandle) -> ShouldRender {
        let old_state = self.handle.state();
        let new_state = handle.state();

        let should_render = new_state.is_new_orbital(old_state)
            || new_state.quality() != old_state.quality()
            || new_state.supplement() != old_state.supplement();

        self.handle.neq_assign(handle);

        should_render
    }

    fn view(&self) -> Html {
        let state = self.handle.state();
        let supplement = state.supplement();
        if supplement.is_enabled() {
            let title = supplement.to_string();
            let desc = match supplement {
                Visualization::None => "",
                Visualization::RadialWavefunction => DESC.rad_wavefunction,
                Visualization::RadialProbabilityDistribution => DESC.rad_prob_distr,
                Visualization::CrossSectionXY
                | Visualization::CrossSectionYZ
                | Visualization::CrossSectionZX => DESC.cross_section,
                Visualization::Isosurface3D => DESC.isosurface_3d,
            };

            let isosurface_cutoff_text = if supplement == Visualization::Isosurface3D {
                html! {
                    <p>
                        { "Specifically, the cutoff value used is " }
                        <RawSpan
                            inner_html = utils::fmt_scientific_notation(
                                plotters::isosurface_cutoff_heuristic(state.qn()).powi(2),
                                3,
                            )
                        />
                        { "." }
                    </p>
                }
            } else {
                html! {}
            };

            html! {
                <>
                    <h3>{ utils::capitalize_words(&title) }</h3>
                    <p><RawSpan inner_html = desc /></p>
                    { isosurface_cutoff_text }
                    <div class = "visualization" id = Self::ID />
                </>
            }
        } else {
            html! {}
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        if self.handle.state().supplement().is_enabled() {
            self.rerender();
            // Fire resize event since the size of the description may change.
            utils::fire_resize_event();
        }
    }
}

pub(crate) type SupplementalVisualization = SharedStateComponent<SupplementalVisualizationImpl>;
