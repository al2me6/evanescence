use std::borrow::Cow;
use std::fmt::Display;
use std::marker::PhantomData;

use web_sys::HtmlSelectElement;
use yew::prelude::*;

pub trait DropdownItem: Copy + PartialEq + Display + 'static {}
impl<T> DropdownItem for T where T: Copy + PartialEq + Display + 'static {}

pub struct Dropdown<T: DropdownItem> {
    node_ref: NodeRef,
    phantom: PhantomData<T>,
}

#[derive(PartialEq, Properties)]
pub struct DropdownProps<T: DropdownItem> {
    pub id: &'static str,
    pub on_change: Callback<T>,
    pub options: Vec<T>,
    pub selected: T,
    #[prop_or_default]
    pub custom_display: Option<Vec<String>>,
}

impl<T: DropdownItem> Component for Dropdown<T> {
    type Message = usize;
    type Properties = DropdownProps<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
            phantom: PhantomData,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        ctx.props().on_change.emit(ctx.props().options[msg]);
        false
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.node_ref
            .cast::<HtmlSelectElement>()
            .unwrap()
            .set_value(
                &ctx.props()
                    .options
                    .iter()
                    .position(|opt| opt == &ctx.props().selected)
                    .unwrap()
                    .to_string(),
            );
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let onchange = ctx.link().callback(|evt: Event| {
            evt.target_unchecked_into::<HtmlSelectElement>()
                .value()
                .parse::<usize>()
                .unwrap()
        });

        let option = |idx: usize, selected_item: &T| {
            let item = &props.options[idx];
            // Use `Cow` to avoid unnecessary cloning.
            let display = match &props.custom_display {
                Some(custom_display) => Cow::from(&custom_display[idx]),
                None => Cow::from(item.to_string()),
            };
            html! {
                <option selected = { item == selected_item } value = { idx.to_string() }>
                    { display }
                </option>
            }
        };

        html! {
            <select
                ref = { self.node_ref.clone() }
                id = { props.id }
                { onchange }
                aria-label = { props.id }
            >
                { for (0..props.options.len()).map(|idx| option(idx, &props.selected)) }
            </select>
        }
    }
}
