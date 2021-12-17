use web_sys::HtmlInputElement;
use yew::prelude::*;

pub struct Slider {
    /// Store the value internally so that the label can be updated while dragging.
    value: f32,
    node_ref: NodeRef,
}

#[derive(PartialEq, Properties)]
pub struct SliderProps {
    pub id: &'static str,
    pub on_change: Callback<f32>,
    pub min: f32,
    pub value: f32,
    pub max: f32,
    pub step: f32,
    pub value_postfix: String,
}

impl Component for Slider {
    type Message = bool;
    type Properties = SliderProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            value: ctx.props().value,
            node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.value = self
            .node_ref
            .cast::<HtmlInputElement>()
            .unwrap()
            .value()
            .parse()
            .unwrap();
        if msg {
            // true = mouse released
            ctx.props().on_change.emit(self.value);
        }
        !msg
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.value = ctx.props().value;
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        html! {
            <div class = "slider" id = { props.id }>
                <input
                    ref = { self.node_ref.clone() }
                    type = "range"
                    oninput = { ctx.link().callback(|_| false) }
                    onchange = { ctx.link().callback(|_| true) }
                    min = { props.min.to_string() }
                    value = { self.value.to_string() }
                    max = { props.max.to_string() }
                    step = { props.step.to_string() }
                    aria-label = { props.id }
                />
                <p class = "slider-label">
                    { format!("{:.1}{}", self.value, props.value_postfix) }
                </p>
            </div>
        }
    }
}
