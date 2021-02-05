use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yewtil::NeqAssign;

pub(crate) struct Tooltip {
    props: TooltipProps,
}

#[derive(Clone, PartialEq, Properties)]
pub(crate) struct TooltipProps {
    pub(crate) text: String,
    pub(crate) tooltip: String,
}

impl Component for Tooltip {
    type Message = ();
    type Properties = TooltipProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! {
            <span class = "tooltip">
                { &self.props.text }
                <span class = "description">{ &self.props.tooltip }</span>
            </span>
        }
    }
}
