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
        html! {
            <div class="header">
                <a href="/">{"About"}</a>
                <a href="/catalog">{"Catalog"}</a>
            </div>
        }
    }
}
