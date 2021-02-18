#![recursion_limit = "1024"]
#[warn(clippy::pedantic)]
#[allow(
    clippy::default_trait_access // Triggered by yew's proc macros.
)]
pub(crate) mod components;
pub(crate) mod descriptions;
pub(crate) mod plotly;
pub(crate) mod plotters;
pub(crate) mod state;
pub(crate) mod ui;
pub(crate) mod utils;

use std::panic;

use pkg_version::{pkg_version_major, pkg_version_minor, pkg_version_patch};
use wasm_bindgen::prelude::*;
use yew::{html, start_app, Component, ComponentLink, Html, ShouldRender};
use yew_state::SharedStateComponent;
use yewtil::NeqAssign;

use crate::state::StateHandle;
use crate::ui::{Controls, InfoPanel, PointillistVisualization, SupplementalVisualization};

/// Maximum value of the principal quantum number `n` that is exposed.
pub(crate) const MAX_N: u32 = 8;

pub(crate) const VER_MAJOR: u32 = pkg_version_major!();
pub(crate) const VER_MINOR: u32 = pkg_version_minor!();
pub(crate) const VER_PATCH: u32 = pkg_version_patch!();

pub(crate) const REPO: &str = "https://github.com/al2me6/evanescence";
pub(crate) const BENCHMARKS_URL: &str = "https://al2me6.github.io/evanescence/dev/bench";

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
        const SEPARATOR: &str = " | ";

        let footer = html! {
            <footer>
                <p>
                    { format!("Evanescence v{}.{}.{}", VER_MAJOR, VER_MINOR, VER_PATCH) }
                    { SEPARATOR }
                    <a href = format!("{}/blob/master/evanescence_web/changelog.md", REPO) >
                        { "Changelog" }
                    </a>
                    { SEPARATOR }
                    <a href = REPO>{ "Source" }</a>
                    { SEPARATOR }
                    <a href = BENCHMARKS_URL>{ "Benchmarks" }</a>
                </p>
            </footer>
        };

        html! {
            <>
            <main>
                <PointillistVisualization/>
            </main>
            <aside id = "sidebar">
                <h1>{ "Hydrogenic Orbitals" }</h1>
                <Controls/>
                <InfoPanel/>
                <SupplementalVisualization/>
                { footer }
            </aside>
            </>
        }
    }
}

type Main = SharedStateComponent<MainImpl>;

#[wasm_bindgen(start)]
pub fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let config = if cfg!(debug_assertions) {
        wasm_logger::Config::default()
    } else {
        wasm_logger::Config::new(log::Level::Warn)
    };
    wasm_logger::init(config);

    start_app::<Main>();
}
