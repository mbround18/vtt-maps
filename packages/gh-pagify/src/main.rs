mod api;
mod components;
mod entities;
mod pages;
mod utils;

use yew::prelude::*;
use yew_router::prelude::*;

use components::Header;
use pages::{Catalog, NotFound, ReadMe, Route};

fn switch(routes: Route) -> Html {
    match routes {
        Route::ReadMe => html! { <ReadMe /> },
        Route::Catalog => html! { <Catalog /> },
        Route::NotFound => html! { <NotFound /> },
    }
}
#[function_component]
fn App() -> Html {
    html! {
        <>
            <Header />
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
