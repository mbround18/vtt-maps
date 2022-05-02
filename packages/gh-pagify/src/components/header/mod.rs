use crate::Route;
use yew::prelude::*;
use yew_router::prelude::*;

// Definition
pub struct Header;

// Props
#[derive(PartialEq, Properties)]
pub struct HeaderProps;

// Implementation
impl Component for Header {
    type Message = ();
    type Properties = HeaderProps;

    // On Initialize
    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }
    // demo

    // On Render
    fn view(&self, _ctx: &Context<Self>) -> Html {
        let readme = Route::ReadMe;
        let catalog = Route::Catalog;
        html! {
            <div class="header">
                <Link<Route> to={readme}>{ "About" }</Link<Route>>
                <Link<Route> to={catalog}>{ "Catalog" }</Link<Route>>
            </div>
        }
    }
}
