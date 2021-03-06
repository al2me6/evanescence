use std::borrow::Cow;
use std::fmt::Display;
use std::str::FromStr;

use web_sys::HtmlSelectElement;
use yew::prelude::*;
use yewtil::NeqAssign;

pub(crate) trait DropdownItem: Copy + PartialEq + Display + 'static {}
impl<T> DropdownItem for T where T: Copy + PartialEq + Display + 'static {}

pub(crate) struct Dropdown<T: DropdownItem> {
    link: ComponentLink<Self>,
    props: DropdownProps<T>,
}

#[derive(Clone, PartialEq, Properties)]
pub(crate) struct DropdownProps<T: DropdownItem> {
    pub(crate) id: String,
    pub(crate) onchange: Callback<T>,
    pub(crate) options: Vec<T>,
    pub(crate) selected: T,
    #[prop_or_default]
    pub(crate) custom_display: Option<Vec<String>>,
}

impl<T: DropdownItem> Component for Dropdown<T> {
    type Message = String;
    type Properties = DropdownProps<T>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.props
            .onchange
            .emit(self.props.options[usize::from_str(&msg).unwrap()]);
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        fn into_select_element(data: ChangeData) -> HtmlSelectElement {
            match data {
                ChangeData::Select(select) => select,
                _ => unreachable!(),
            }
        }

        let option = |idx: usize, selected_item: &T| {
            let item = &self.props.options[idx];
            // Use `Cow` to avoid unnecessary cloning.
            let display = match &self.props.custom_display {
                Some(custom_display) => Cow::from(&custom_display[idx]),
                None => Cow::from(item.to_string()),
            };
            html! {
                <option selected = item == selected_item value = idx.to_string()>{ display }</option>
            }
        };

        html! {
            <select
                id = self.props.id
                onchange = self.link.callback(|data: ChangeData| into_select_element(data).value())
                aria-label = self.props.id
            >
                { for (0..self.props.options.len()).map(|idx| option(idx, &self.props.selected)) }
            </select>
        }
    }
}
