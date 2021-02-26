use strum::IntoEnumIterator;
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew_state::SharedStateComponent;
use yewtil::NeqAssign;

use crate::components::TabBar;
use crate::state::{Mode, State, StateHandle};
use crate::utils::fire_resize_event;

pub(crate) struct ModeBarImpl {
    handle: StateHandle,
}

impl Component for ModeBarImpl {
    type Message = ();
    type Properties = StateHandle;

    fn create(handle: StateHandle, _link: ComponentLink<Self>) -> Self {
        Self { handle }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, handle: StateHandle) -> ShouldRender {
        self.handle.neq_assign(handle)
    }

    fn view(&self) -> Html {
        let handle = &self.handle;
        let state = handle.state();

        let set_mode = |state: &mut State, mode| {
            state.set_mode(mode);
            fire_resize_event();
        };

        html! {
            <TabBar<Mode>
                id = "mode"
                onchange = handle.reduce_callback_with(set_mode)
                modes = Mode::iter().collect::<Vec<_>>()
                selected = state.mode()
            />
        }
    }
}

pub(crate) type ModeBar = SharedStateComponent<ModeBarImpl>;
