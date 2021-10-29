use web_sys::HtmlElement;
use yew::prelude::*;
use yewtil::NeqAssign;

use crate::utils::CowStr;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct RawProps {
    pub inner_html: CowStr,
    #[prop_or_default]
    pub class: CowStr,
}

macro_rules! raw_element_type {
    ($name:ident, $element:ident) => {
        pub struct $name {
            props: RawProps,
            node_ref: NodeRef,
        }

        impl Component for $name {
            type Message = ();
            type Properties = RawProps;

            fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
                Self {
                    props,
                    node_ref: NodeRef::default(),
                }
            }

            fn update(&mut self, _: Self::Message) -> ShouldRender {
                false
            }

            fn change(&mut self, props: Self::Properties) -> ShouldRender {
                self.props.neq_assign(props)
            }

            fn view(&self) -> Html {
                html! {
                    <$element
                        class = format!("raw {}", self.props.class)
                        ref = self.node_ref.clone()
                    />
                }
            }

            fn rendered(&mut self, _first_render: bool) {
                self.node_ref
                    .cast::<HtmlElement>()
                    .unwrap()
                    .set_inner_html(&self.props.inner_html);
            }
        }
    };
}

raw_element_type!(RawSpan, span);
raw_element_type!(RawDiv, div);
