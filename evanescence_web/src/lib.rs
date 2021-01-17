#![recursion_limit = "512"]

pub(crate) mod components;
pub(crate) mod evanescence_bridge;
pub(crate) mod plotly;
pub(crate) mod ui;

use std::cell::RefCell;
use std::panic;
use std::rc::Rc;

use evanescence_core::monte_carlo::Quality;
use evanescence_core::orbital::{self, Orbital, Qn};
use pkg_version::{pkg_version_major, pkg_version_minor, pkg_version_patch};
use wasm_bindgen::prelude::*;
use yew::{html, App, Component, ComponentLink, Html, ShouldRender};

use crate::components::RawSpan;
use crate::ui::{Controls, PointillistVisualization};

/// Maximum value of the principal quantum number `n` that is exposed.
pub(crate) const MAX_N: u32 = 8;

pub(crate) const VER_MAJOR: u32 = pkg_version_major!();
pub(crate) const VER_MINOR: u32 = pkg_version_minor!();
pub(crate) const VER_PATCH: u32 = pkg_version_patch!();

#[derive(PartialEq, Eq, Default)]
pub(crate) struct AppState {
    qn: Qn,
    quality: Quality,
}

struct Main {
    link: ComponentLink<Self>,
    state: Rc<RefCell<AppState>>,
}

impl Component for Main {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            state: Rc::new(RefCell::new(AppState::default())),
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
            <main>
                <PointillistVisualization id = "pointillist" state=self.state.clone() />
            </main>
            <aside>
                <h1>{"Hydrogenic Orbitals"}</h1>
                <Controls
                    onchange = self.link.callback(|_| ())
                    state = self.state.clone()
                />
                <p>
                    {"Viewing orbital: "}
                    <RawSpan inner_html = orbital::Real::name(self.state.borrow().qn) />
                </p>
                <p>{ format!(
                        "Quality: {} ({} points)",
                        self.state.borrow().quality,
                        self.state.borrow().quality as usize
                    )
                }</p>
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

#[wasm_bindgen(start)]
pub fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());

    App::<Main>::new().mount_to_body();
}
