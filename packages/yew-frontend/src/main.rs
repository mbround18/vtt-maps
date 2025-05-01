mod api;
mod components;
mod entities;
mod utils;

use console_error_panic_hook::set_once;
use tracing_wasm::set_as_global_default;
use yew::prelude::*;
use yew_router::prelude::*;
mod pages;
use components::header::Header;
use pages::{Catalog, MapDetail, NotFound, ReadMe, Route};

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
        Route::ReadMe => html! { <ReadMe /> },
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
