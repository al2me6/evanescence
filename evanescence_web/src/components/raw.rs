use web_sys::HtmlElement;
use yew::{html, Component, ComponentLink, Html, NodeRef, Properties, ShouldRender};
use yewtil::NeqAssign;

#[derive(Clone, Debug, PartialEq, Properties)]
pub(crate) struct RawProps {
    pub(crate) inner_html: String,
}

macro_rules! raw_element_type {
    ($name:ident, $element:ident) => {
        pub(crate) struct $name {
            inner_html: String,
            node_ref: NodeRef,
        }

        impl Component for $name {
            type Message = ();
            type Properties = RawProps;

            fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
                Self {
                    inner_html: props.inner_html,
                    node_ref: NodeRef::default(),
                }
            }

            fn update(&mut self, _: Self::Message) -> ShouldRender {
                false
            }

            fn change(&mut self, props: Self::Properties) -> ShouldRender {
                self.inner_html.neq_assign(props.inner_html)
            }

            fn view(&self) -> Html {
                html! { <$element class = "raw" ref = self.node_ref.clone() /> }
            }

            fn rendered(&mut self, _first_render: bool) {
                self.node_ref
                    .cast::<HtmlElement>()
                    .unwrap()
                    .set_inner_html(&self.inner_html);
            }
        }
    };
}

raw_element_type!(RawSpan, span);
raw_element_type!(RawDiv, div);
