use yew::prelude::*;
use crate::api::local::get_catalog;


pub struct Catalog {
    content: Html,
}

pub enum CatalogMessage {
    SetContent(Html),
}

impl Component for Catalog {
    type Message = CatalogMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async move {
            let body = get_catalog().await;
            let content: Html = Html::from_html_unchecked(body.into());
            CatalogMessage::SetContent(content)
        });
        Self {
            content: html! { "loading..."},
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CatalogMessage::SetContent(content) => {
                self.content = content;
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
       html! {
           <>
            <div id="catalog">{self.content.clone()}</div>
           </>
           }

    }
}

