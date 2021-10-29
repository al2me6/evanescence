#![feature(drain_filter)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yewdux::prelude::*;
use yewtil::NeqAssign;

pub mod ui;

use evanescence_web::components::raw::RawDiv;
use evanescence_web::components::Window;
use evanescence_web::state::AppDispatch;

use crate::ui::{
    Controls, InfoPanel, ModeBar, PointillistVisualization, SupplementalVisualization,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REPO: &str = env!("CARGO_PKG_REPOSITORY");

pub const HELP_HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/help.html"));

struct MainImpl {
    dispatch: AppDispatch,
    resize_handler: Closure<dyn Fn()>,
}

impl MainImpl {
    const SIDEBAR_ID: &'static str = "sidebar";

    fn viewport_change_handler() {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();
        let scroll_y = window.scroll_y().unwrap();
        let sidebar = document.get_element_by_id(Self::SIDEBAR_ID).unwrap();

        // If the offset is not zero, then the flexbox must have wrapped. If so, we activate the
        // vertical layout. There does not appear to be a way to detect wrapping in CSS.
        if sidebar.get_bounding_client_rect().y() + scroll_y > 0.0 {
            body.class_list().add_1("vertical-layout").unwrap();
        } else {
            body.class_list().remove_1("vertical-layout").unwrap();
        }

        // HACK: Force the page height to be the same as `innerHeight`. This prevents browsers'
        // navigation bars from overlapping with content. Ideally this would be done in CSS, but
        // there doesn't appear to be a great way. `height: -webkit-fill-available;` appears to
        // behave erratically as of March 2021.
        body.style()
            .set_property(
                "height",
                &format!("{}px", window.inner_height().unwrap().as_f64().unwrap()),
            )
            .unwrap();
    }
}

impl Component for MainImpl {
    type Message = ();
    type Properties = AppDispatch;

    fn create(dispatch: AppDispatch, _link: ComponentLink<Self>) -> Self {
        Self {
            dispatch,
            resize_handler: Closure::wrap(Box::new(Self::viewport_change_handler)),
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, dispatch: AppDispatch) -> ShouldRender {
        self.dispatch.neq_assign(dispatch);
        false
    }

    fn view(&self) -> Html {
        let footer = html! {
            <footer>
                <p>{ format!("Evanescence {}", VERSION) }</p>
                <span>
                    <a href = format!("{}/blob/master/CHANGELOG.md", REPO) target = "_blank">
                        { "Change Log" }
                    </a>
                </span>
                <span><a href = REPO target = "_blank">{ "Source" }</a></span>
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

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let window = web_sys::window().unwrap();
            for event in ["resize", "orientationchange"] {
                window
                    .add_event_listener_with_callback(
                        event,
                        self.resize_handler.as_ref().unchecked_ref(),
                    )
                    .unwrap();
            }
            Self::viewport_change_handler();
        }
    }
}

type Main = WithDispatch<MainImpl>;

#[allow(clippy::missing_panics_doc)]
fn main() {
    std::panic::set_hook(Box::new(|info| {
        let window = web_sys::window().unwrap();

        // Clear state to prevent the page from crashing again upon reload.
        #[cfg(feature = "persistent")]
        window.session_storage().unwrap().unwrap().clear().unwrap();

        console_error_panic_hook::hook(info);
        let payload = match info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => s.as_ref(),
                None => "<unknown error>",
            },
        };
        window
            .alert_with_message(&format!(
                "Evanescence encountered a serious error: {}.\nPlease refresh the page.",
                payload,
            ))
            .unwrap();
    }));

    #[cfg(debug_assertions)]
    let config = wasm_logger::Config::default();
    #[cfg(not(debug_assertions))]
    let config = wasm_logger::Config::new(log::Level::Info);
    wasm_logger::init(config);

    App::<Main>::new().mount_to_body();
}
