use evanescence_core::monte_carlo::MonteCarlo;
use evanescence_core::orbital;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_state::{SharedState, SharedStateComponent};
use yewtil::NeqAssign;

use crate::evanescence_bridge;
use crate::plotly::config::{Config, ModeBarButtons};
use crate::plotly::layout::{Layout, Scene};
use crate::plotly::Plotly;
use crate::StateHandle;

#[derive(Clone, PartialEq, Properties)]
pub(crate) struct VisualizationProps {
    pub(crate) id: String,
    #[prop_or_default]
    pub(crate) handle: StateHandle,
}

impl SharedState for VisualizationProps {
    type Handle = StateHandle;

    fn handle(&mut self) -> &mut Self::Handle {
        self.handle.handle()
    }
}

pub(crate) struct PointillistVisualizationImpl {
    props: VisualizationProps,
    has_rendered: bool,
}

impl PointillistVisualizationImpl {
    fn render_plot(&mut self) {
        let state = self.props.handle.state();
        let trace = evanescence_bridge::into_scatter3d_real(orbital::Real::monte_carlo_simulate(
            state.qn,
            state.quality,
        ));

        if self.has_rendered {
            Plotly::delete_trace(&self.props.id, 0);
            Plotly::add_trace(&self.props.id, trace);
        } else {
            Plotly::react(
                &self.props.id,
                &[trace],
                Layout {
                    ui_revision: &self.props.id,
                    scene: Some(Scene::default()),
                    ..Default::default()
                },
                Config {
                    mode_bar_buttons_to_remove: &[
                        ModeBarButtons::ResetCameraLastSave3d,
                        ModeBarButtons::HoverClosest3d,
                    ],
                    ..Default::default()
                },
            );
        }
    }
}

impl Component for PointillistVisualizationImpl {
    type Message = ();
    type Properties = VisualizationProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            props,
            has_rendered: false,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props.neq_assign(props) {
            self.render_plot();
        }
        false
    }

    fn rendered(&mut self, _first_render: bool) {
        self.render_plot();
        self.has_rendered = true;
    }

    fn view(&self) -> Html {
        html! {
            <div class="visualization" id = self.props.id />
        }
    }
}

pub(crate) type PointillistVisualization = SharedStateComponent<PointillistVisualizationImpl>;
