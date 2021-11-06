use evanescence_web::components::raw::RawSpan;
use evanescence_web::components::window::OpenButton;
use evanescence_web::components::Window;
use evanescence_web::plotly::config::ModeBarButtons;
use evanescence_web::plotly::{Config, Plotly};
use evanescence_web::plotters::supplemental as plot;
use evanescence_web::state::{AppDispatch, State, Visualization};
use evanescence_web::{time_scope, utils};
use gloo::utils::document;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlElement;
use yew::prelude::*;
use yewdux::prelude::*;
use yewtil::NeqAssign;

use super::descriptions::DESC;

pub struct SupplementalVisualizationImpl {
    link: ComponentLink<Self>,
    dispatch: AppDispatch,
}

impl SupplementalVisualizationImpl {
    const ID_FULLSCREEN_CONTAINER: &'static str = "supplemental-fullscreen-content";
    const ID_INFO_PLOT: &'static str = "supplemental-content";
    const ID_PLACEHOLDER: &'static str = "supplemental-placeholder";
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
        let content = document().get_element_by_id(Self::ID_INFO_PLOT).unwrap();

        // Prevent the background from resizing itself by creating a placeholder element that is
        // the same height as the content while maximizing.
        let placeholder = document()
            .get_element_by_id(Self::ID_PLACEHOLDER)
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        if is_open {
            // Sum the full heights (including padding, but NOT margins!) of all child elements.
            let children = content.children();
            let content_height = (0..children.length())
                .map(|idx| children.item(idx).unwrap())
                .map(|elem| elem.scroll_height())
                .sum::<i32>();
            placeholder
                .style()
                .set_property("height", &format!("{}px", content_height))
                .unwrap();
        } else {
            placeholder.style().set_property("height", "0px").unwrap();
        }

        let target_container = document()
            .get_element_by_id(if is_open {
                Self::ID_FULLSCREEN_CONTAINER
            } else {
                Self::ID_WRAPPER
            })
            .unwrap();
        target_container.append_child(&content).unwrap();

        Plotly::resize(Self::ID_PLOT);
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
            let title = utils::capitalize_words(supplement.to_string());
            let desc = match supplement {
                Visualization::None => unreachable!(),
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

            let open_button_gen = |cb| html! {
                <button
                    type = "button"
                    id = "supplemental-maximize-btn"
                    class = "window-button"
                    title = "Enlarge"
                    onclick = cb
                >
                    { "\u{200B}" } // U+200B ZERO WIDTH SPACE for alignment purposes.
                </button>
            };

            html! {
                <div id = Self::ID_WRAPPER>
                    <div id = "supplemental-title">
                        <h3>{ &title }</h3>
                        <Window
                            title = title
                            id = "supplemental-fullscreen-window"
                            content_id = Self::ID_FULLSCREEN_CONTAINER
                            open_button = OpenButton::Custom(open_button_gen)
                            on_toggle = self.link.callback(|is_open| is_open)
                        />
                    </div>
                    <div id = Self::ID_PLACEHOLDER />
                    <div id = Self::ID_INFO_PLOT>
                        // HACK: Wrap the text elements in another div so that the height of their
                        // margins is included when summing the `scrollHeight`s of the parent div.
                        <div>
                            <p><RawSpan inner_html = desc /></p>
                            { isosurface_cutoff_text }
                        </div>
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
            // Resize since the size of the description may change.
            Plotly::resize(Self::ID_PLOT);
        }
    }
}

pub type SupplementalVisualization = WithDispatch<SupplementalVisualizationImpl>;
