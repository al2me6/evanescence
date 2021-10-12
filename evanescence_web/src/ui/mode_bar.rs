use strum::IntoEnumIterator;
use yew::prelude::*;
use yewdux::prelude::*;
use yewtil::NeqAssign;

use crate::components::TabBar;
use crate::state::{AppDispatch, Mode, State};
use crate::utils;

pub(crate) struct ModeBarImpl {
    dispatch: AppDispatch,
}

impl Component for ModeBarImpl {
    type Message = ();
    type Properties = AppDispatch;

    fn create(dispatch: AppDispatch, _link: ComponentLink<Self>) -> Self {
        Self { dispatch }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, dispatch: AppDispatch) -> ShouldRender {
        self.dispatch.neq_assign(dispatch)
    }

    fn view(&self) -> Html {
        let dispatch = &self.dispatch;
        let state = dispatch.state();

        let set_mode = |state: &mut State, mode| {
            state.set_mode(mode);
            if state.supplement().is_enabled() {
                utils::fire_resize_event();
            }
        };

        html! {
            <TabBar<Mode>
                id = "mode"
                on_change = dispatch.reduce_callback_with(set_mode)
                modes = Mode::iter().collect::<Vec<_>>()
                selected = state.mode()
            />
        }
    }
}

pub(crate) type ModeBar = WithDispatch<ModeBarImpl>;
