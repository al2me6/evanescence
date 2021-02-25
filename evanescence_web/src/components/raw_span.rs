use web_sys::HtmlSpanElement;
use yew::{html, Component, ComponentLink, Html, NodeRef, Properties, ShouldRender};
use yewtil::NeqAssign;

#[derive(Clone, Debug, PartialEq, Properties)]
pub(crate) struct RawHtmlProps {
    pub(crate) inner_html: String,
}

pub(crate) struct RawSpan {
    inner_html: String,
    span_ref: NodeRef,
}

impl Component for RawSpan {
    type Message = ();
    type Properties = RawHtmlProps;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {
            inner_html: props.inner_html,
            span_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.inner_html.neq_assign(props.inner_html)
    }

    fn view(&self) -> Html {
        html! { <span ref = self.span_ref.clone() /> }
    }

    fn rendered(&mut self, _first_render: bool) {
        self.span_ref
            .cast::<HtmlSpanElement>()
            .unwrap()
            .set_inner_html(&self.inner_html);
    }
}
