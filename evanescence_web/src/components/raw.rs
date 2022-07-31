use web_sys::HtmlElement;
use yew::prelude::*;

#[derive(PartialEq, Eq, Properties)]
pub struct RawProps {
    pub inner_html: String,
    #[prop_or_default]
    pub class: String,
}

macro_rules! raw_element_type {
    ($name:ident, $element:ident) => {
        pub struct $name {
            node_ref: NodeRef,
        }

        impl Component for $name {
            type Message = ();
            type Properties = RawProps;

            fn create(_ctx: &Context<Self>) -> Self {
                Self {
                    node_ref: NodeRef::default(),
                }
            }

            fn view(&self, ctx: &Context<Self>) -> Html {
                html! {
                    <$element
                        ref = { self.node_ref.clone() }
                        class = { format!("raw {}", ctx.props().class) }
                    />
                }
            }

            fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
                self.node_ref
                    .cast::<HtmlElement>()
                    .unwrap()
                    .set_inner_html(&ctx.props().inner_html);
            }
        }
    };
}

raw_element_type!(RawSpan, span);
raw_element_type!(RawDiv, div);
