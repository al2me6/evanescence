use web_sys::HtmlElement;
use yew::prelude::*;
use yewtil::NeqAssign;

use crate::utils::CowStr;

pub struct Window {
    link: ComponentLink<Self>,
    props: WindowProps,
    node_ref: NodeRef,
}

pub enum WindowMsg {
    Open,
    Close,
}

#[derive(Clone, PartialEq, Properties)]
pub struct WindowProps {
    pub title: CowStr,
    #[prop_or_default]
    pub content_id: CowStr,
    #[prop_or_default]
    pub open_button_hover: CowStr,
    pub open_button_text: CowStr,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub on_toggle: Option<Callback<bool>>,
}

impl Component for Window {
    type Message = WindowMsg;
    type Properties = WindowProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.node_ref
            .cast::<HtmlElement>()
            .unwrap()
            .set_attribute(
                "window_vis_",
                match msg {
                    WindowMsg::Open => "visible",
                    WindowMsg::Close => "hidden",
                },
            )
            .unwrap();
        // Disable scrolling for the body in CSS.
        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        match msg {
            WindowMsg::Open => body.class_list().add_1("window-open").unwrap(),
            WindowMsg::Close => body.class_list().remove_1("window-open").unwrap(),
        }
        if let Some(cb) = self.props.on_toggle.as_ref() {
            cb.emit(matches!(msg, WindowMsg::Open));
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! {
            <>
            <button
                type = "button"
                class = "window-button"
                title = &self.props.open_button_hover
                onclick = self.link.callback(|_| WindowMsg::Open)
            >
                { &self.props.open_button_text }
            </button>
            <div class = "window-bg" ref = self.node_ref.clone()>
                <div class = "window-container">
                    <div class = "window-header">
                        <h1>{ &self.props.title }</h1>
                        <button
                            type = "button"
                            class = "window-button window-close-button"
                            onclick = self.link.callback(|_| WindowMsg::Close)
                            title = "Close"
                        />
                    </div>
                    <div id = &self.props.content_id class = "window-content">
                        { self.props.children.clone() }
                    </div>
                </div>
            </div>
            </>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.update(WindowMsg::Close);
        }
    }
}
