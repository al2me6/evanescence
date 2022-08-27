use std::mem;
use std::rc::Rc;

use evanescence_web::components::raw::RawSpan;
use evanescence_web::components::window::OpenButton;
use evanescence_web::components::Window;
use evanescence_web::plotly::config::ModeBarButtons;
use evanescence_web::plotly::{Config, Plotly};
use evanescence_web::plotters::{supplemental as plot, ISOSURFACE_CUTOFF};
use evanescence_web::state::{State, StateDispatch, Visualization};
use evanescence_web::{time_scope, utils};
use gloo::utils::document;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlElement;
use yew::prelude::*;

use super::descriptions::DESC;

pub struct SupplementalVisualization {
    current_state: Rc<State>,
}

impl SupplementalVisualization {
    const ID_FULLSCREEN_CONTAINER: &'static str = "supplemental-fullscreen-content";
    const ID_INFO_PLOT: &'static str = "supplemental-content";
    const ID_PLACEHOLDER: &'static str = "supplemental-placeholder";
    const ID_PLOT: &'static str = "supplemental";
    const ID_WRAPPER: &'static str = "supplemental-panel";

    fn rerender(&mut self, state: &State) {
        let renderer: fn(&State) -> (Vec<JsValue>, JsValue) = match state.supplement() {
            Visualization::None => return, // No need to render.
            Visualization::RadialWavefunction
            | Visualization::RadialProbabilityDistribution
            | Visualization::CumulativeRadialDistribution => plot::radial,
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

        let (traces, layout) = renderer(state);
        Plotly::react(
            Self::ID_PLOT,
            traces.into_boxed_slice(),
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

impl Component for SupplementalVisualization {
    type Message = bool;
    type Properties = StateDispatch;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            current_state: ctx.props().state(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, is_open: Self::Message) -> bool {
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
                .set_property("height", &format!("{content_height}px"))
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

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let old_state = &mem::replace(&mut self.current_state, ctx.props().state());
        let new_state = &self.current_state;

        new_state.is_new_orbital(old_state)
            || new_state.supplement() != old_state.supplement()
            || (new_state.quality() != old_state.quality())
                && !matches!(
                    old_state.supplement(),
                    Visualization::RadialWavefunction | Visualization::CumulativeRadialDistribution
                )
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = ctx.props().state();
        let supplement = state.supplement();

        if !supplement.is_enabled() {
            return html!();
        }

        let title = utils::capitalize_words(supplement.to_string());
        let desc = match supplement {
            Visualization::None => unreachable!(),
            Visualization::RadialWavefunction => DESC.rad_wavefunction,
            Visualization::RadialProbabilityDistribution => DESC.rad_prob_distr,
            Visualization::CumulativeRadialDistribution => DESC.rad_cumulative,
            Visualization::WavefunctionXY
            | Visualization::WavefunctionYZ
            | Visualization::WavefunctionZX => DESC.cross_section_wavefunction,
            Visualization::ProbabilityDensityXY
            | Visualization::ProbabilityDensityYZ
            | Visualization::ProbabilityDensityZX => DESC.cross_section_prob_density,
            Visualization::Isosurface3D => DESC.isosurface_3d,
        };

        let isosurface_cutoff_text = html! {
            if supplement == Visualization::Isosurface3D {
                <p>
                    { "The surfaces drawn (|ψ|² = " }
                    <RawSpan inner_html = {
                        utils::fmt_scientific_notation(state.isosurface_cutoff().powi(2), 2)
                    } />
                    { format!(") enclose ~{:.0}% of all probability density.", ISOSURFACE_CUTOFF * 100.) }
                </p>
            }
        };

        let open_button_gen = |onclick| {
            html! {
                <button
                    type = "button"
                    id = "supplemental-maximize-btn"
                    class = "window-button"
                    title = "Enlarge"
                    { onclick }
                >
                    { "\u{200B}" } // U+200B ZERO WIDTH SPACE for alignment purposes.
                </button>
            }
        };

        html! {
            <div id = {Self::ID_WRAPPER}>
                <div id = "supplemental-title">
                    <h3>{ &title }</h3>
                    <Window
                        {title}
                        id = "supplemental-fullscreen-window"
                        content_id = { Self::ID_FULLSCREEN_CONTAINER }
                        open_button = { OpenButton::Custom(open_button_gen) }
                        on_toggle = { ctx.link().callback(std::convert::identity) }
                    />
                </div>
                <div id = { Self::ID_PLACEHOLDER } />
                <div id = { Self::ID_INFO_PLOT }>
                    // HACK: Wrap the text elements in another div so that the height of their
                    // margins is included when summing the `scrollHeight`s of the parent div.
                    <div>
                        <p><RawSpan inner_html = { desc } /></p>
                        { isosurface_cutoff_text }
                    </div>
                    <div class = "visualization" id = { Self::ID_PLOT } />
                </div>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        if ctx.props().state().supplement().is_enabled() {
            self.rerender(&ctx.props().state());
            // Resize since the size of the description may change.
            Plotly::resize(Self::ID_PLOT);
        }
    }
}
