#![feature(drain_filter)]

use evanescence_web::components::raw::RawDiv;
use evanescence_web::components::window::OpenButton;
use evanescence_web::components::Window;
use evanescence_web::state::AppDispatch;
use gloo::events::EventListener;
use gloo::storage::{SessionStorage, Storage};
use gloo::utils::{body, document, window};
use yew::prelude::*;
use yewdux::prelude::*;

pub mod ui;

use crate::ui::{
    Controls, InfoPanel, ModeBar, PointillistVisualization, SupplementalVisualization,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REPO: &str = env!("CARGO_PKG_REPOSITORY");

pub static HELP_HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/help.html"));

struct MainImpl {
    _resize_listener: EventListener,
    _orientation_listener: EventListener,
}

impl MainImpl {
    const SIDEBAR_ID: &'static str = "sidebar";

    fn viewport_change_handler() {
        let scroll_y = window().scroll_y().unwrap();
        let sidebar = document().get_element_by_id(Self::SIDEBAR_ID).unwrap();

        // If the offset is not zero, then the flexbox must have wrapped. If so, we activate the
        // vertical layout. There does not appear to be a way to detect wrapping in CSS.
        if sidebar.get_bounding_client_rect().y() + scroll_y > 0.0 {
            body().class_list().add_1("vertical-layout").unwrap();
        } else {
            body().class_list().remove_1("vertical-layout").unwrap();
        }

        // HACK: Force the page height to be the same as `innerHeight`. This prevents browsers'
        // navigation bars from overlapping with content. Ideally this would be done in CSS, but
        // there doesn't appear to be a great way. `height: -webkit-fill-available;` appears to
        // behave erratically as of March 2021.
        body()
            .style()
            .set_property(
                "height",
                &format!("{}px", window().inner_height().unwrap().as_f64().unwrap()),
            )
            .unwrap();
    }
}

impl Component for MainImpl {
    type Message = ();
    type Properties = AppDispatch;

    fn create(_ctx: &Context<Self>) -> Self {
        let resize_listener =
            EventListener::new(&window(), "resize", |_| Self::viewport_change_handler());
        let orientation_listener = EventListener::new(&window(), "orientationchange", |_| {
            Self::viewport_change_handler()
        });
        Self {
            _resize_listener: resize_listener,
            _orientation_listener: orientation_listener,
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let footer = html! {
            <footer>
                <p>{ format!("Evanescence {VERSION}") }</p>
                <span>
                    <a href = { format!("{REPO}/blob/master/CHANGELOG.md") } target = "_blank">
                        { "Change Log" }
                    </a>
                </span>
                <span><a href = { REPO } target = "_blank">{ "Source" }</a></span>
            </footer>
        };

        html! {
            <>
            <main>
                <PointillistVisualization/>
            </main>
            <div id = { Self::SIDEBAR_ID }>
                <header>
                    <div id = "title-and-help-btn">
                        <h1>{ "Hydrogenic Orbitals" }</h1>
                        <Window
                            title = "Help"
                            id = "help-window"
                            content_id = "help-content"
                            open_button = { OpenButton::Text('?', Some("Help")) }
                        >
                            <RawDiv inner_html = { HELP_HTML } />
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

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            Self::viewport_change_handler();
        }
    }
}

type Main = WithDispatch<MainImpl>;

#[allow(clippy::missing_panics_doc)]
fn main() {
    #[cfg(debug_assertions)]
    let config = wasm_logger::Config::default();
    #[cfg(not(debug_assertions))]
    let config = wasm_logger::Config::new(log::Level::Info);
    wasm_logger::init(config);

    yew::set_custom_panic_hook(Box::new(|info| {
        // Clear state to prevent the page from crashing again upon reload.
        #[cfg(feature = "persistent")]
        SessionStorage::clear();

        console_error_panic_hook::hook(info);
        let payload = match info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => s.as_ref(),
                None => "<unknown error>",
            },
        };
        gloo::dialogs::alert(&format!(
            "Evanescence encountered a serious error: \n{payload}.\n\nPlease refresh the page.",
        ));
    }));

    yew::start_app::<Main>();
}
