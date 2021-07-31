use yew::prelude::*;
use yewtil::NeqAssign;

use super::raw::RawSpan;
use crate::utils::CowStr;

pub(crate) struct Tooltip {
    props: TooltipProps,
}

#[derive(Clone, PartialEq, Properties)]
pub(crate) struct TooltipProps {
    pub(crate) text: CowStr,
    pub(crate) tooltip: CowStr,
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
                <RawSpan inner_html = &self.props.text />
                <RawSpan class = "description" inner_html = &self.props.tooltip />
            </span>
        }
    }
}
