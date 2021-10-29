use std::fmt::Display;
use std::str::FromStr;

use yew::prelude::*;
use yewtil::NeqAssign;

use crate::utils::CowStr;

pub trait TabBarItem: Copy + PartialEq + Display + 'static {}
impl<T> TabBarItem for T where T: Copy + PartialEq + Display + 'static {}

pub struct TabBar<T: TabBarItem> {
    link: ComponentLink<Self>,
    props: TabBarProps<T>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct TabBarProps<T: TabBarItem> {
    pub id: CowStr,
    pub on_change: Callback<T>,
    pub modes: Vec<T>,
    pub selected: T,
}

impl<T: TabBarItem> Component for TabBar<T> {
    type Message = String;
    type Properties = TabBarProps<T>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.props
            .on_change
            .emit(self.props.modes[usize::from_str(&msg).unwrap()]);
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        fn into_value(data: ChangeData) -> String {
            match data {
                ChangeData::Value(value) => value,
                _ => unreachable!(),
            }
        }

        let tab = |idx: usize, selected_mode: &T| {
            let mode = &self.props.modes[idx];

            // Note that this is implemented in a rather sleazy way. We put the radio button's
            // label into the `label_text_` attribute, which is then grabbed by the CSS and put
            // into the `::after` pseudo-element of the `input` element. This way we can change
            // the style of the label based on the radio button's selection state directly in CSS.
            html! {
                <input
                    type = "radio"
                    name = &self.props.id
                    checked = mode == selected_mode
                    value = idx.to_string()
                    label_text_ = mode.to_string()
                    onchange = self.link.callback(into_value)
                    aria-label = mode.to_string()
                />
            }
        };

        html! {
            <div class = "tab-bar">
                { for (0..self.props.modes.len()).map(|idx| tab(idx, &self.props.selected)) }
            </div>
        }
    }
}
