use crate::api::local::get_readme;
use yew::{html, Component, Context, Html, Properties};

#[derive(PartialEq, Properties)]
pub struct Props;

pub enum Msg {
    SetReadme(String),
}

pub struct ReadMe {
    readme: String,
}

impl Component for ReadMe {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async move {
            let body = get_readme().await;
            Msg::SetReadme(body)
        });
        Self {
            readme: String::from("# VTT-Maps"),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetReadme(readme) => {
                self.readme = markdown::to_html(&readme);
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let div = gloo_utils::document().create_element("div").unwrap();
        div.set_inner_html(&self.readme);
        div.set_id("readme");
        div.set_class_name("card");
        html! {
            <>{ Html::VRef(div.into()) }</>
        }
    }
}
