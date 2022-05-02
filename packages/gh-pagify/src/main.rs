mod api;
mod components;
mod entities;
mod pages;
mod utils;

use yew::prelude::*;
use yew_router::prelude::*;

use components::Header;
use pages::{Catalog, MapAssetView, NotFound, ReadMe, Route};

fn switch(routes: &Route) -> Html {
    match routes {
        Route::ReadMe => html! { <ReadMe /> },
        Route::Catalog => html! { <Catalog /> },
        Route::MapAssetView { id } => {
            let identifier = id.to_string();
            return html! { <MapAssetView id={identifier} /> };
        }
        Route::NotFound => html! { <NotFound /> },
    }
}

#[function_component(Main)]
fn app() -> Html {
    html! {
        <>
            <Header />
            <BrowserRouter>
                <Switch<Route> render={Switch::render(switch)} />
            </BrowserRouter>
        </>
    }
}

fn main() {
    yew::start_app::<Main>();
}
