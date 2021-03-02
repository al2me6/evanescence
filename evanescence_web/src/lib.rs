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
use yew::{html, start_app, Component, ComponentLink, Html, ShouldRender};
use yew_state::SharedStateComponent;
use yewtil::NeqAssign;

use crate::components::raw::RawDiv;
use crate::components::Window;
use crate::state::StateHandle;
use crate::ui::{
    Controls,
    InfoPanel,
    ModeBar,
    PointillistVisualization,
    SupplementalVisualization,
};

/// Maximum value of the principal quantum number `n` that is exposed.
pub(crate) const MAX_N: u32 = 8;

pub(crate) const VER_MAJOR: u32 = pkg_version_major!();
pub(crate) const VER_MINOR: u32 = pkg_version_minor!();
pub(crate) const VER_PATCH: u32 = pkg_version_patch!();

pub(crate) const REPO: &str = "https://github.com/al2me6/evanescence";
pub(crate) const BENCHMARKS_URL: &str = "https://al2me6.github.io/evanescence/dev/bench";

pub(crate) const HELP_HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/help.html"));

struct MainImpl {
    handle: StateHandle,
    resize_handler: Closure<dyn Fn()>,
}

impl MainImpl {
    const SIDEBAR_ID: &'static str = "sidebar";

    fn resize_handler() {
        let document = web_sys::window().unwrap().document().unwrap();
        let sidebar = document.get_element_by_id(Self::SIDEBAR_ID).unwrap();

        // If the offset is not zero, then the flexbox must have wrapped. If so, we activate the
        // mobile layout.
        if sidebar.get_bounding_client_rect().y() > 0.0 {
            sidebar.class_list().add_1("mobile-layout").unwrap();
        } else {
            sidebar.class_list().remove_1("mobile-layout").unwrap();
        }
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
        let footer = html! {
            <footer>
                <p>{ format!("Evanescence v{}.{}.{}", VER_MAJOR, VER_MINOR, VER_PATCH) }</p>
                <span><a href = format!("{}/blob/master/changelog.md", REPO) >{ "Changelog" }</a></span>
                <span><a href = REPO>{ "Source" }</a></span>
                <span><a href = BENCHMARKS_URL>{ "Benchmarks" }</a></span>
            </footer>
        };

        html! {
            <>
            <main>
                <PointillistVisualization/>
            </main>
            <div id = Self::SIDEBAR_ID>
                <header>
                    <div id = "title-and-help-btn">
                        <h1>{ "Hydrogenic Orbitals" }</h1>
                        <Window
                            title = "Help"
                            content_id = "help-window"
                            open_button_text = "?"
                            open_button_hover = "Help"
                        >
                            <RawDiv inner_html = HELP_HTML />
                        </Window>
                    </div>
                    <ModeBar/> // Mutates state!
                </header>
                <Controls/> // Mutates state!
                <InfoPanel/>
                <SupplementalVisualization/>
                { footer }
            </div>
            </>
        }
    }
}

type Main = SharedStateComponent<MainImpl>;

#[wasm_bindgen(start)]
pub fn run() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    #[cfg(debug_assertions)]
    let config = wasm_logger::Config::default();
    #[cfg(not(debug_assertions))]
    let config = wasm_logger::Config::new(log::Level::Warn);
    wasm_logger::init(config);

    start_app::<Main>();
}
