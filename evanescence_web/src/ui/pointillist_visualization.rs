use std::cell::RefCell;
use std::rc::Rc;

use evanescence_core::monte_carlo::MonteCarlo;
use evanescence_core::orbital;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yewtil::NeqAssign;

use crate::evanescence_bridge;
use crate::plotly::layout::{Layout, Scene};
use crate::plotly::{Config, ModeBarButtons, Plotly};
use crate::AppState;

#[derive(Clone, PartialEq, Properties)]
pub(crate) struct VisualizationProps {
    pub(crate) id: String,
    pub(crate) state: Rc<RefCell<AppState>>,
}

pub(crate) struct PointillistVisualization {
    props: VisualizationProps,
    has_rendered: bool,
}

impl PointillistVisualization {
    fn render_plot(&mut self) {
        let state = self.props.state.borrow();
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

impl Component for PointillistVisualization {
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
        self.props.neq_assign(props);
        self.render_plot();
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
