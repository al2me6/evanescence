use std::collections::HashMap;
use std::fmt::Display;

use web_sys::HtmlSelectElement;
use yew::{html, Callback, ChangeData, Component, ComponentLink, Html, Properties, ShouldRender};
use yewtil::NeqAssign;

pub(crate) trait DropdownItem: Copy + PartialEq + Display + 'static {}
impl<T> DropdownItem for T where T: Copy + PartialEq + Display + 'static {}

pub(crate) struct Dropdown<T: DropdownItem> {
    link: ComponentLink<Self>,
    props: ControlsProps<T>,
    item_strings: HashMap<String, T>,
}

#[derive(Clone, PartialEq, Properties)]
pub(crate) struct ControlsProps<T: DropdownItem> {
    #[prop_or_default]
    pub(crate) id: String,
    pub(crate) onchange: Callback<T>,
    pub(crate) options: Vec<T>,
    pub(crate) selected: T,
}

impl<T: DropdownItem> Dropdown<T> {
    fn rebuild_hashmap(&mut self) {
        self.item_strings.clear();
        // Borrow `self.item_strings` mutably to get around the borrow checker...
        let item_strings = &mut self.item_strings;
        // ...here because we also borrow `self.props` mutably...
        self.props.options.iter().for_each(|option| {
            // ...and attempt to access `self.item_strings` while `self.props` is still borrowed.
            item_strings.insert(option.to_string(), *option);
        });
    }
}

impl<T: DropdownItem> Component for Dropdown<T> {
    type Message = String;
    type Properties = ControlsProps<T>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut dropdown = Self {
            link,
            props,
            item_strings: HashMap::new(),
        };
        dropdown.rebuild_hashmap();
        dropdown
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.props.onchange.emit(self.item_strings[&msg]);
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props.neq_assign(props) {
            self.rebuild_hashmap();
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        fn into_select_element(data: ChangeData) -> HtmlSelectElement {
            match data {
                ChangeData::Select(select) => select,
                _ => unreachable!(),
            }
        }

        fn option<T: Display + PartialEq>(value: &T, selected_value: &T) -> Html {
            html! { <option selected = value == selected_value >{ value }</option> }
        }

        html! {
            <select
                id = self.props.id
                onchange=self.link.callback(|data: ChangeData| into_select_element(data).value())
            >
                { for self.props.options.iter().map(|opt| option(opt, &self.props.selected)) }
            </select>
        }
    }
}
