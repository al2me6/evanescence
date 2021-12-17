use yew::function_component;
use yew::prelude::*;

use super::raw::RawSpan;

#[derive(PartialEq, Properties)]
pub struct TooltipProps {
    pub text: String,
    pub tooltip: String,
}

#[function_component(Tooltip)]
pub fn tooltip(props: &TooltipProps) -> Html {
    html! {
        <span class = "tooltip">
            <RawSpan inner_html = { props.text.clone() } />
            <RawSpan class = "description" inner_html = { props.tooltip.clone() } />
        </span>
    }
}
