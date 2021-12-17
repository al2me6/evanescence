use web_sys::HtmlInputElement;
use yew::prelude::*;

use super::Tooltip;

pub struct CheckBox {
    node_ref: NodeRef,
}

#[derive(PartialEq, Properties)]
pub struct CheckBoxProps {
    pub id: &'static str,
    pub on_change: Callback<bool>,
    pub checked: bool,
    pub label: String,
    #[prop_or_default]
    pub tooltip: Option<&'static str>,
}

impl Component for CheckBox {
    type Message = ();
    type Properties = CheckBoxProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, _msg: Self::Message) -> bool {
        let state = self.node_ref.cast::<HtmlInputElement>().unwrap().checked();
        ctx.props().on_change.emit(state);
        false
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.node_ref
            .cast::<HtmlInputElement>()
            .unwrap()
            .set_checked(ctx.props().checked);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let label_text = html! {
            if let Some(tooltip) = props.tooltip {
                <Tooltip text = { props.label.clone() } { tooltip } />
            } else {
                <span>{ props.label.clone() }</span>
            }
        };
        html! {
            <label class = "checkbox">
                <input
                    ref = { self.node_ref.clone() }
                    type = "checkbox"
                    id = { props.id }
                    onchange = { ctx.link().callback(|_| ()) }
                    checked = { props.checked }
                    aria-label = { props.label.clone() }
                />
                { label_text }
            </label>
        }
    }
}
