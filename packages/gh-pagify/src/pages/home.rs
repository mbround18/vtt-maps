use crate::components::markdown::Markdown;
use yew::prelude::*;

// Definition
pub struct Home {}

// Implementation
impl Component for Home {
    type Message = ();
    type Properties = ();

    // On Initialize
    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    // When state change
    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    // On Render
    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <Markdown url={"/README.md"} />
            </div>
        }
    }
}
