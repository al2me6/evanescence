use wasm_bindgen::JsValue;
use web_sys::HtmlElement;
use yew::{html, Component, ComponentLink, Html, NodeRef, ShouldRender};
use yew_state::SharedStateComponent;
use yewtil::NeqAssign;

use crate::evanescence_bridge::{plot_cross_section, plot_isosurface_3d, plot_radial};
use crate::plotly::config::ModeBarButtons;
use crate::plotly::{Config, Plotly};
use crate::state::{State, StateHandle, Visualization};
use crate::utils::capitalize_words;

pub(crate) struct SupplementalVisualizationImpl {
    handle: StateHandle,
    title_ref: NodeRef,
    div_ref: NodeRef,
}

impl SupplementalVisualizationImpl {
    const ID: &'static str = "supplemental";

    fn rerender(&mut self) {
        let state = self.handle.state();

        for element in [&self.title_ref, &self.div_ref].iter() {
            let style = element.cast::<HtmlElement>().unwrap().style();
            let display = match state.extra_visualization {
                Visualization::None => "none",
                _ => "block",
            };
            style.set_property("display", display).unwrap();
        }

        let renderer: fn(&State) -> (JsValue, JsValue) = match state.extra_visualization {
            Visualization::None => return, // No need to render.
            Visualization::RadialWavefunction
            | Visualization::RadialProbability
            | Visualization::RadialProbabilityDistribution => plot_radial,
            Visualization::CrossSectionXY
            | Visualization::CrossSectionYZ
            | Visualization::CrossSectionZX => plot_cross_section,
            Visualization::Isosurface3D => plot_isosurface_3d,
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
            div_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, handle: StateHandle) -> ShouldRender {
        let diff = self.handle.state().diff(handle.state());
        self.handle.neq_assign(handle);
        if diff.extra_visualization || diff.qn_or_quality {
            self.rerender();
        }
        diff.extra_visualization // We need to rerender the title if the visualization type changes.
    }

    fn rendered(&mut self, _first_render: bool) {
        self.rerender();
    }

    fn view(&self) -> Html {
        let title = self.handle.state().extra_visualization.to_string();
        html! {
            <>
                // The title can't be in the same div as the visualization because that somehow
                // breaks Plotly's responsive mode detection (reverts to hard-coded 450px
                // vertical size).
                <h3 ref = self.title_ref.clone() >{ capitalize_words(&title) }</h3>
                <div ref = self.div_ref.clone() class = "visualization" id = Self::ID />
            </>
        }
    }
}

pub(crate) type SupplementalVisualization = SharedStateComponent<SupplementalVisualizationImpl>;
