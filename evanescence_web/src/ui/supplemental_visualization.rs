use wasm_bindgen::JsValue;
use web_sys::HtmlElement;
use yew::{html, Component, ComponentLink, Html, NodeRef, ShouldRender};
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
    title_ref: NodeRef,
    desc_ref: NodeRef,
    plot_ref: NodeRef,
}

impl SupplementalVisualizationImpl {
    const ID: &'static str = "supplemental";

    fn rerender(&mut self) {
        let state = self.handle.state();

        for element in [&self.title_ref, &self.desc_ref, &self.plot_ref].iter() {
            let style = element.cast::<HtmlElement>().unwrap().style();
            let display = match state.supplement {
                Visualization::None => "none",
                _ => "block",
            };
            style.set_property("display", display).unwrap();
        }

        let renderer: fn(&State) -> (JsValue, JsValue) = match state.supplement {
            Visualization::None => return, // No need to render.
            Visualization::RadialWavefunction
            | Visualization::RadialProbabilityDensity
            | Visualization::RadialProbabilityDistribution => plot::radial,
            Visualization::CrossSectionXY
            | Visualization::CrossSectionYZ
            | Visualization::CrossSectionZX => plot::cross_section,
            Visualization::Isosurface3D => plot::isosurface_3d,
        };

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
        Self {
            handle,
            title_ref: NodeRef::default(),
            desc_ref: NodeRef::default(),
            plot_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, handle: StateHandle) -> ShouldRender {
        let diff = self.handle.state().diff(handle.state());
        self.handle.neq_assign(handle);
        diff.supplement || diff.qn_or_quality_or_mode
    }

    fn rendered(&mut self, _first_render: bool) {
        self.rerender();
        fire_resize_event();
    }

    fn view(&self) -> Html {
        let extra_visualization = self.handle.state().supplement;
        let title = extra_visualization.to_string();
        let desc = match extra_visualization {
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
                // The title can't be in the same div as the visualization because that somehow
                // breaks Plotly's responsive mode detection (reverts to hard-coded 450px
                // vertical size).
                <h3 ref = self.title_ref.clone() >{ capitalize_words(&title) }</h3>
                <p ref = self.desc_ref.clone() >{ desc }</p>
                <div ref = self.plot_ref.clone() class = "visualization" id = Self::ID />
            </>
        }
    }
}

pub(crate) type SupplementalVisualization = SharedStateComponent<SupplementalVisualizationImpl>;
