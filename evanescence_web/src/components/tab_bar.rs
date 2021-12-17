use std::fmt::Display;
use std::marker::PhantomData;

use web_sys::HtmlInputElement;
use yew::prelude::*;

pub trait TabBarItem: Copy + PartialEq + Display + 'static {}
impl<T> TabBarItem for T where T: Copy + PartialEq + Display + 'static {}
pub struct TabBar<T: TabBarItem> {
    _item_ty: PhantomData<T>,
}

#[derive(PartialEq, Properties)]
pub struct TabBarProps<T: TabBarItem> {
    pub id: &'static str,
    pub on_change: Callback<T>,
    pub modes: Vec<T>,
    pub selected: T,
}

impl<T: TabBarItem> Component for TabBar<T> {
    type Message = usize;
    type Properties = TabBarProps<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            _item_ty: PhantomData,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        ctx.props().on_change.emit(ctx.props().modes[msg]);
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let tab = |idx: usize, selected_mode: &T| {
            let mode = &props.modes[idx];
            let onchange = ctx.link().callback(|evt: Event| {
                evt.target_unchecked_into::<HtmlInputElement>()
                    .value()
                    .parse::<usize>()
                    .unwrap()
            });

            // Note that this is implemented in a rather sleazy way. We put the radio button's
            // label into the `label_text_` attribute, which is then grabbed by the CSS and put
            // into the `::after` pseudo-element of the `input` element. This way we can change
            // the style of the label based on the radio button's selection state directly in CSS.
            html! {
                <input
                    type = "radio"
                    name = { props.id }
                    checked = {mode == selected_mode }
                    value = { idx.to_string() }
                    label_text_ = { mode.to_string() }
                    { onchange }
                    aria-label = { mode.to_string() }
                />
            }
        };

        html! {
            <div class = "tab-bar">
                { for (0..props.modes.len()).map(|idx| tab(idx, &props.selected)) }
            </div>
        }
    }
}
