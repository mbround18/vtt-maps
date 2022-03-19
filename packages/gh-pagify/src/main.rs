mod api;
mod components;
mod entities;
mod pages;

use pages::{Catalogue, Home};
use components::navigation::Navigation;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
enum MainRoute {
    #[at("/")]
    Home,
    #[at("/catalogue")]
    Catalogue,
    #[not_found]
    #[at("/settings/404")]
    NotFound,
}

fn switch_main(route: &MainRoute) -> Html {
    match route {
        MainRoute::Home => html! {
            <Home />
        },
        MainRoute::Catalogue => html! {
            <Catalogue />
        },
        MainRoute::NotFound => html! {<h1>{"Not Found"}</h1>},
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <>
            <BrowserRouter>
                <Navigation />
                <Switch<MainRoute> render={Switch::render(switch_main)} />
            </BrowserRouter>
        </>
    }
}

fn main() {
    yew::start_app::<App>();
}
