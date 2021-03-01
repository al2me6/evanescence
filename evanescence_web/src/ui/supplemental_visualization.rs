use wasm_bindgen::JsValue;
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew_state::SharedStateComponent;
use yewtil::NeqAssign;

use crate::descriptions::DESC;
use crate::plotly::config::ModeBarButtons;
use crate::plotly::{Config, Plotly};
use crate::plotters::supplemental as plot;
use crate::state::{State, StateHandle, Visualization};
use crate::utils::{capitalize_words, fire_resize_event};

pub(crate) struct SupplementalVisualizationImpl {
    handle: StateHandle,
}

impl SupplementalVisualizationImpl {
    const ID: &'static str = "supplemental";

    fn rerender(&mut self) {
        let state = self.handle.state();

        let renderer: fn(&State) -> (JsValue, JsValue) = match state.supplement() {
            Visualization::None => return, // No need to render.
            Visualization::RadialWavefunction
            | Visualization::RadialProbabilityDensity
            | Visualization::RadialProbabilityDistribution => plot::radial,
            Visualization::CrossSectionXY
            | Visualization::CrossSectionYZ
            | Visualization::CrossSectionZX => plot::cross_section,
            Visualization::Isosurface3D => plot::isosurface_3d,
        };

        log::debug!("Rerendering {:?}.", state.supplement());

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
        let supplement = self.handle.state().supplement();
        if supplement.is_enabled() {
            let title = supplement.to_string();
            let desc = match supplement {
                Visualization::None => "",
                Visualization::RadialWavefunction => DESC.rad_wavefunction,
                Visualization::RadialProbabilityDensity => DESC.rad_prob_density,
                Visualization::RadialProbabilityDistribution => DESC.rad_prob_distr,
                Visualization::CrossSectionXY
                | Visualization::CrossSectionYZ
                | Visualization::CrossSectionZX => DESC.cross_section,
                Visualization::Isosurface3D => DESC.isosurface_3d,
            };
            html! {
                <>
                    <h3>{ capitalize_words(&title) }</h3>
                    <p>{ desc }</p>
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
            fire_resize_event();
        }
    }
}

pub(crate) type SupplementalVisualization = SharedStateComponent<SupplementalVisualizationImpl>;
