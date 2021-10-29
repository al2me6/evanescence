use yew::prelude::*;
use yewtil::NeqAssign;

use crate::utils::CowStr;

pub struct Button {
    link: ComponentLink<Self>,
    props: ButtonProps,
}

#[derive(Clone, PartialEq, Properties)]
pub struct ButtonProps {
    pub id: CowStr,
    pub enabled: bool,
    pub on_click: Callback<()>,
    pub text: CowStr,
    pub hover: CowStr,
}

impl Component for Button {
    type Message = ();
    type Properties = ButtonProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        self.props.on_click.emit(());
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! {
            <button
                type = "button"
                class = "button"
                title = &self.props.hover
                onclick = self.link.callback(|_| ())
                disabled = !self.props.enabled
            >
                { &self.props.text }
            </button>
        }
    }
}
