#![recursion_limit = "512"]

pub(crate) mod components;
pub(crate) mod evanescence_bridge;
pub(crate) mod plotly;
pub(crate) mod ui;

use std::panic;

use evanescence_core::monte_carlo::Quality;
use evanescence_core::orbital::{self, Orbital, Qn};
use pkg_version::{pkg_version_major, pkg_version_minor, pkg_version_patch};
use wasm_bindgen::prelude::*;
use yew::{html, start_app, Component, ComponentLink, Html, ShouldRender};
use yew_state::{SharedHandle, SharedStateComponent};
use yewtil::NeqAssign;

use crate::components::RawSpan;
use crate::ui::{Controls, PointillistVisualization};

/// Maximum value of the principal quantum number `n` that is exposed.
pub(crate) const MAX_N: u32 = 8;

pub(crate) const VER_MAJOR: u32 = pkg_version_major!();
pub(crate) const VER_MINOR: u32 = pkg_version_minor!();
pub(crate) const VER_PATCH: u32 = pkg_version_patch!();

#[derive(Clone, PartialEq, Eq, Default)]
pub(crate) struct State {
    qn: Qn,
    quality: Quality,
}

pub(crate) type StateHandle = SharedHandle<State>;

struct MainImpl {
    handle: StateHandle,
}

impl Component for MainImpl {
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
        let state = self.handle.state();
        html! {
            <>
            <main>
                <PointillistVisualization id = "pointillist"/>
            </main>
            <aside>
                <h1>{"Hydrogenic Orbitals"}</h1>
                <Controls/>
                <p>
                    {"Viewing orbital: "}
                    <RawSpan inner_html = orbital::Real::name(state.qn)/>
                </p>
                <p>{ format!("Quality: {} ({} points)", state.quality, state.quality as usize) }</p>
                <footer>
                    <p>
                        {format!("Evanescence v{}.{}.{}", VER_MAJOR, VER_MINOR, VER_PATCH)}
                        {" | "}
                        <a href="https://github.com/al2me6/evanescence">{"Source"}</a>
                        {" | "}
                        <a href="https://al2me6.github.io/evanescence/dev/bench">{"Benchmarks"}</a>
                    </p>
                </footer>
            </aside>
            </>
        }
    }
}

type Main = SharedStateComponent<MainImpl>;

#[wasm_bindgen(start)]
pub fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());

    start_app::<Main>();
}
