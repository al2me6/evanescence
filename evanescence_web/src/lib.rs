#![recursion_limit = "1024"]
#![feature(drain_filter)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_precision_loss, // We work with smaller values, so this should not be a concern.
    clippy::default_trait_access, // Triggered by yew's proc macros.
    clippy::filter_map, // Semantics.
    clippy::needless_pass_by_value, // Triggered by wasm-bindgen macro.
    clippy::non_ascii_literal, // Unicode support is expected.
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
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
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
    resize_handler: Closure<dyn Fn()>,
}

impl MainImpl {
    const SIDEBAR_ID: &'static str = "sidebar";

    fn resize_handler() {
        let sidebar = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(Self::SIDEBAR_ID)
            .unwrap();
        let style = sidebar.dyn_ref::<HtmlElement>().unwrap().style();

        // If the offset is not zero, then the flexbox must have wrapped. In this case, we do not
        // need an additional scrollbar in the sidebar (the whole page already scrolls). However,
        // on desktop, only the sidebar should scroll.
        let offset = sidebar.get_bounding_client_rect().y();
        let (max_height, overflow_y) = if offset > 0.0 {
            ("unset", "unset")
        } else {
            ("100vh", "auto")
        };
        style.set_property("max-height", max_height).unwrap();
        style.set_property("overflow-y", overflow_y).unwrap();
    }
}

impl Component for MainImpl {
    type Message = ();
    type Properties = StateHandle;

    fn create(handle: StateHandle, _link: ComponentLink<Self>) -> Self {
        Self {
            handle,
            resize_handler: Closure::wrap(Box::new(Self::resize_handler)),
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, handle: StateHandle) -> ShouldRender {
        self.handle.neq_assign(handle);
        false
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            web_sys::window()
                .unwrap()
                .add_event_listener_with_callback(
                    "resize",
                    self.resize_handler.as_ref().unchecked_ref(),
                )
                .unwrap();
            Self::resize_handler();
        }
    }

    fn view(&self) -> Html {
        const SEPARATOR: &str = " | ";

        let footer = html! {
            <footer>
                <p>
                    { format!("Evanescence v{}.{}.{}", VER_MAJOR, VER_MINOR, VER_PATCH) }
                    { SEPARATOR }
                    <a href = format!("{}/blob/master/changelog.md", REPO) >
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
            <aside id = Self::SIDEBAR_ID>
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
