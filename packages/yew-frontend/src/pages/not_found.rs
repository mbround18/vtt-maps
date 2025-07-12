use yew::{Component, Context, Html, Properties, html};

#[derive(PartialEq, Properties)]
pub struct Props;

pub struct NotFound;

impl Component for NotFound {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! { <div id="not-found" class="card">{
             "The page you were looking for is not found!"
        }</div> }
    }
}
