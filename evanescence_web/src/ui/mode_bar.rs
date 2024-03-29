use evanescence_web::components::TabBar;
use evanescence_web::state::{Mode, State, StateDispatch};
use evanescence_web::utils;
use strum::IntoEnumIterator;
use yew::function_component;
use yew::prelude::*;
use yewdux::prelude::*;

#[function_component(ModeBar)]
pub fn mode_bar(props: &StateDispatch) -> Html {
    let state = props.state();

    let set_mode = |state: &mut State, mode| {
        state.set_mode(mode);
        if state.supplement().is_enabled() {
            utils::fire_resize_event();
        }
    };

    html! {
        <TabBar<Mode>
            id = "mode"
            on_change = { props.reduce_callback_with(set_mode) }
            modes = { Mode::iter().collect::<Vec<_>>() }
            selected = { state.mode() }
        />
    }
}
