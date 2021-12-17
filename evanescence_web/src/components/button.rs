#![allow(clippy::if_not_else)] // Spurious.

use yew::function_component;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ButtonProps {
    pub id: &'static str,
    pub enabled: bool,
    pub on_click: Callback<()>,
    pub text: String,
    pub hover: String,
}

#[function_component(Button)]
pub fn button(props: &ButtonProps) -> Html {
    html! {
        <button
            type = "button"
            class = "button"
            id = { props.id }
            title = { props.hover.clone() }
            onclick = { props.on_click.reform(|_| ()) }
            disabled = { !props.enabled }
        >
            { &props.text }
        </button>
    }
}
