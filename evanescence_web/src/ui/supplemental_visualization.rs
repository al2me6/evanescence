use evanescence_web::components::raw::RawSpan;
use evanescence_web::components::Window;
use evanescence_web::plotly::config::ModeBarButtons;
use evanescence_web::plotly::{Config, Plotly};
use evanescence_web::plotters::supplemental as plot;
use evanescence_web::state::{AppDispatch, State, Visualization};
use evanescence_web::{time_scope, utils};
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yewdux::prelude::*;
use yewtil::NeqAssign;

use super::descriptions::DESC;

pub struct SupplementalVisualizationImpl {
    link: ComponentLink<Self>,
    dispatch: AppDispatch,
}

impl SupplementalVisualizationImpl {
    const ID_CONTENT: &'static str = "supplemental-content";
    const ID_FULLSCREEN_CONTAINER: &'static str = "supplemental-fullscreen";
    const ID_PLOT: &'static str = "supplemental";
    const ID_WRAPPER: &'static str = "supplemental-panel";

    fn rerender(&mut self) {
        let state = self.dispatch.state();

        let renderer: fn(&State) -> (JsValue, JsValue) = match state.supplement() {
            Visualization::None => return, // No need to render.
            Visualization::RadialWavefunction | Visualization::RadialProbabilityDistribution => {
                plot::radial
            }
            Visualization::WavefunctionXY
            | Visualization::WavefunctionYZ
            | Visualization::WavefunctionZX => plot::cross_section,
            Visualization::ProbabilityDensityXY
            | Visualization::ProbabilityDensityYZ
            | Visualization::ProbabilityDensityZX => plot::cross_section_prob_density,
            Visualization::Isosurface3D => plot::isosurface_3d,
        };

        log::debug!("Rerendering {:?}.", state.supplement());

        let _timer = time_scope!(
            "[{}][{}] Render {:?} supplement",
            state.debug_description(),
            state.quality(),
            state.supplement(),
        );

        let (trace, layout) = renderer(state);
        Plotly::react(
            Self::ID_PLOT,
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
        );
    }
}

impl Component for SupplementalVisualizationImpl {
    type Message = bool;
    type Properties = AppDispatch;

    fn create(dispatch: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, dispatch }
    }

    fn update(&mut self, is_open: Self::Message) -> ShouldRender {
        let document = web_sys::window().unwrap().document().unwrap();
        let content = document.get_element_by_id(Self::ID_CONTENT).unwrap();
        let target_container = document
            .get_element_by_id(if is_open {
                Self::ID_FULLSCREEN_CONTAINER
            } else {
                Self::ID_WRAPPER
            })
            .unwrap();
        target_container.append_child(&content).unwrap();
        utils::fire_resize_event();
        false
    }

    fn change(&mut self, dispatch: Self::Properties) -> ShouldRender {
        let old_state = self.dispatch.state();
        let new_state = dispatch.state();

        let should_render = new_state.is_new_orbital(old_state)
            || new_state.supplement() != old_state.supplement()
            || (new_state.quality() != old_state.quality()) && !old_state.supplement().is_radial();

        self.dispatch.neq_assign(dispatch);

        should_render
    }

    fn view(&self) -> Html {
        let state = self.dispatch.state();
        let supplement = state.supplement();
        if supplement.is_enabled() {
            let title = supplement.to_string();
            let desc = match supplement {
                Visualization::None => "",
                Visualization::RadialWavefunction => DESC.rad_wavefunction,
                Visualization::RadialProbabilityDistribution => DESC.rad_prob_distr,
                Visualization::WavefunctionXY
                | Visualization::WavefunctionYZ
                | Visualization::WavefunctionZX => DESC.cross_section_wavefunction,
                Visualization::ProbabilityDensityXY
                | Visualization::ProbabilityDensityYZ
                | Visualization::ProbabilityDensityZX => DESC.cross_section_prob_density,
                Visualization::Isosurface3D => DESC.isosurface_3d,
            };

            let isosurface_cutoff_text = if supplement == Visualization::Isosurface3D {
                html! {
                    <p>
                        { "Specifically, the cutoff value used is " }
                        <RawSpan inner_html = utils::fmt_scientific_notation(
                            state.isosurface_cutoff().powi(2),
                            3,
                        ) />
                        { "." }
                    </p>
                }
            } else {
                html!()
            };

            html! {
                <div id = Self::ID_WRAPPER>
                    <div id = "supplemental-title">
                        <h3>{ utils::capitalize_words(&title) }</h3>
                        <Window
                            title = utils::capitalize_words(&title)
                            content_id = Self::ID_FULLSCREEN_CONTAINER
                            open_button_text = "+"
                            open_button_hover = "Enlarge"
                            on_toggle = self.link.callback(|is_open| is_open)
                        />
                    </div>
                    <div id = Self::ID_CONTENT>
                        <p><RawSpan inner_html = desc /></p>
                        { isosurface_cutoff_text }
                        <div class = "visualization" id = Self::ID_PLOT />
                    </div>
                </div>
            }
        } else {
            html!()
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        if self.dispatch.state().supplement().is_enabled() {
            self.rerender();
            // Fire resize event since the size of the description may change.
            utils::fire_resize_event();
        }
    }
}

pub type SupplementalVisualization = WithDispatch<SupplementalVisualizationImpl>;
