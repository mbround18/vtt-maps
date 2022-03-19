use yew::prelude::*;
use yew_router::prelude::*;
use crate::MainRoute;

#[function_component(Navigation)]
pub fn nav_items() -> Html {
    let history = use_history().unwrap();

    let go_home_button = {
        let history = history.clone();
        let onclick = Callback::once(move |_| history.push(MainRoute::Home));
        html! {
            <button {onclick}>{"Home"}</button>
        }
    };

    let go_to_catalogue = {
        let history = history.clone();
        let onclick = Callback::once(move |_| history.push(MainRoute::Catalogue));
        html! {
            <button {onclick}>{"Catalogue"}</button>
        }
    };

    html! {
        <div class="navigation">
            {go_home_button}
            {go_to_catalogue}
        </div>
    }
}
