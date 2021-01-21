use web_sys::HtmlInputElement;
use yew::{html, Callback, Component, ComponentLink, Html, NodeRef, Properties, ShouldRender};
use yewtil::NeqAssign;

pub(crate) struct CheckBox {
    link: ComponentLink<Self>,
    props: CheckBoxProps,
    state: bool,
    node_ref: NodeRef,
}

#[derive(Clone, PartialEq, Properties)]
pub(crate) struct CheckBoxProps {
    pub(crate) id: String,
    pub(crate) onchange: Callback<bool>,
    pub(crate) initial_state: bool,
    pub(crate) label: String,
}

impl Component for CheckBox {
    type Message = ();
    type Properties = CheckBoxProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let state = props.initial_state;
        Self {
            link,
            props,
            state,
            node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        self.state = self.node_ref.cast::<HtmlInputElement>().unwrap().checked();
        self.props.onchange.emit(self.state);
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! {
            <label class = "checkbox">
                <input
                    ref = self.node_ref.clone(),
                    type = "checkbox",
                    id = self.props.id,
                    onchange = self.link.callback(|_| ()),
                    checked = self.state
                />
                <span>{ &self.props.label }</span>
            </label>
        }
    }
}
