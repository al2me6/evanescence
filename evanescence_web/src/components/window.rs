use gloo::utils::body;
use web_sys::HtmlElement;
use yew::prelude::*;

pub struct Window {
    node_ref: NodeRef,
}

#[derive(Clone, PartialEq, Eq)]
pub enum OpenButton {
    /// (character on button, optional hover info).
    Text(char, Option<&'static str>),
    /// Function returning a `button` element whose `onclick` is set to the callback passed in.
    Custom(fn(Callback<MouseEvent>) -> Html),
}

#[derive(PartialEq, Eq)]
pub enum WindowMsg {
    Open,
    Close,
}

#[derive(PartialEq, Properties)]
pub struct WindowProps {
    pub title: String,
    pub id: &'static str,
    #[prop_or_default]
    pub content_id: &'static str,
    pub open_button: OpenButton,
    #[prop_or_default]
    pub on_toggle: Option<Callback<bool>>,
    #[prop_or_default]
    pub children: Children,
}

impl Component for Window {
    type Message = WindowMsg;
    type Properties = WindowProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
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
        match msg {
            WindowMsg::Open => body().class_list().add_1("window-open").unwrap(),
            WindowMsg::Close => body().class_list().remove_1("window-open").unwrap(),
        }
        if let Some(ref cb) = ctx.props().on_toggle {
            cb.emit(msg == WindowMsg::Open);
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_open = ctx.link().callback(|_| WindowMsg::Open);
        let open_button = match &ctx.props().open_button {
            OpenButton::Text(ch, hover) => html! {
                <button
                    type = "button"
                    class = "window-button"
                    title = { hover.unwrap_or("") }
                    onclick = { on_open }
                >
                    { ch }
                </button>
            },
            OpenButton::Custom(gen) => gen(on_open),
        };

        html! {
            <>
            { open_button }
            <div id = { ctx.props().id } class = "window-bg" ref = { self.node_ref.clone() }>
                <div class = "window-container">
                    <div class = "window-header">
                        <h1>{ &ctx.props().title }</h1>
                        <button
                            type = "button"
                            class = "window-button window-close-button"
                            onclick = { ctx.link().callback(|_| WindowMsg::Close) }
                            title = "Close"
                        />
                    </div>
                    <div id = { ctx.props().content_id } class = "window-content">
                        { ctx.props().children.clone() }
                    </div>
                </div>
            </div>
            </>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.update(ctx, WindowMsg::Close);
        }
    }
}
