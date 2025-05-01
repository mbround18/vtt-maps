use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::NavBar;
use crate::pages::{Home, MapDetail, NotFound, ReadMe, Route};

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <NavBar />
            <div class="content">
                <Switch<Route> render={switch} />
            </div>
        </BrowserRouter>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::ReadMe => html! { <ReadMe /> },
        Route::Catalog => html! { <Home /> },
        Route::MapDetail { id } => html! { <MapDetail id={id} /> },
        Route::NotFound => html! { <NotFound /> },
    }
}