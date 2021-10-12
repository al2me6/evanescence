use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewtil::NeqAssign;

use crate::utils::CowStr;

pub(crate) struct Slider {
    link: ComponentLink<Self>,
    props: SliderProps,
    node_ref: NodeRef,
}

#[derive(Clone, PartialEq, Properties)]
pub(crate) struct SliderProps {
    pub(crate) id: CowStr,
    pub(crate) on_change: Callback<f32>,
    pub(crate) min: f32,
    pub(crate) value: f32,
    pub(crate) max: f32,
    pub(crate) step: f32,
    pub(crate) value_postfix: CowStr,
}

impl Component for Slider {
    type Message = bool;
    type Properties = SliderProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.props.value = self
            .node_ref
            .cast::<HtmlInputElement>()
            .unwrap()
            .value()
            .parse()
            .unwrap();
        if msg {
            self.props.on_change.emit(self.props.value);
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! {
            <div class = "slider">
                <input
                    ref = self.node_ref.clone()
                    type = "range"
                    id = &self.props.id
                    oninput = self.link.callback(|_| false)
                    onchange = self.link.callback(|_| true)
                    min = self.props.min.to_string()
                    value = self.props.value.to_string()
                    max = self.props.max.to_string()
                    step = self.props.step.to_string()
                    aria-label = &self.props.id
                />
                <p class = "slider-label">
                    { format!("{:.1}{}", self.props.value, self.props.value_postfix) }
                </p>
            </div>
        }
    }
}
