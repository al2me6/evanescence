use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewtil::NeqAssign;

use super::Tooltip;
use crate::utils::CowStr;

pub(crate) struct CheckBox {
    link: ComponentLink<Self>,
    props: CheckBoxProps,
    state: bool,
    node_ref: NodeRef,
}

#[derive(Clone, PartialEq, Properties)]
pub(crate) struct CheckBoxProps {
    pub(crate) id: CowStr,
    pub(crate) on_change: Callback<bool>,
    pub(crate) initial_state: bool,
    pub(crate) label: CowStr,
    #[prop_or_default]
    pub(crate) tooltip: Option<&'static str>,
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
        self.props.on_change.emit(self.state);
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let label_text = if let Some(tooltip) = self.props.tooltip {
            html! {
                <Tooltip text = &self.props.label tooltip = tooltip />
            }
        } else {
            html! { <span>{ &self.props.label }</span> }
        };
        html! {
            <label class = "checkbox">
                <input
                    ref = self.node_ref.clone()
                    type = "checkbox"
                    id = &self.props.id
                    onchange = self.link.callback(|_| ())  // All hail the toilet closure.
                    checked = self.state
                    aria-label = &self.props.label
                />
                { label_text }
            </label>
        }
    }
}
