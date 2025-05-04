mod api;
mod components;
mod entities;
mod utils;

use console_error_panic_hook::set_once;
use tracing_wasm::set_as_global_default;
use yew::prelude::*;
use yew_router::prelude::*;
mod pages;
use crate::api::api::ApiEndpoint;
use crate::pages::MarkdownViewer;
use components::header::Header;
use pages::{Catalog, MapDetail, NotFound, Route};

#[function_component(App)]
fn app() -> Html {
    html! {
        <>
            <Header />
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::ReadMe => html! { <MarkdownViewer
            endpoint={ApiEndpoint::GetMarkdown { path: "readme".to_string() }}
            id={"readme"}
            class={"card"}
            loading_text={"Loading ReadMe content..."}
        /> },
        Route::License => html! { <MarkdownViewer
            endpoint={ApiEndpoint::GetMarkdown { path: "license".to_string() }}
            id={"license"}
            class={"card"}
            loading_text={"Loading License content..."}
        /> },
        Route::Catalog => html! { <Catalog /> },
        Route::MapDetail { id } => html! { <MapDetail id={id} /> },
        Route::NotFound => html! { <NotFound /> },
    }
}

fn main() {
    set_once();
    set_as_global_default();
    yew::Renderer::<App>::new().render();
}
