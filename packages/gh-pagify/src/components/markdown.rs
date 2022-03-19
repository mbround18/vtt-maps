use crate::api::local::get_markdown;
use yew::prelude::*;

pub enum Msg {
    SetMarkdown(String),
}

pub struct Markdown {
    markdown: String,
}

#[derive(Properties, PartialEq)]
pub struct MarkdownProps {
    pub url: String,
}

impl Component for Markdown {
    type Message = Msg;
    type Properties = MarkdownProps;

    fn create(ctx: &Context<Self>) -> Self {
        let url = String::from(&ctx.props().url);
        ctx.link().send_future(async move {
            let body = get_markdown(&url).await;
            Msg::SetMarkdown(body)
        });
        Self {
            markdown: String::from("# VTT-Maps"),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetMarkdown(content) => {
                self.markdown = markdown::to_html(&content);
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let div = gloo_utils::document().create_element("div").unwrap();
        div.set_inner_html(&self.markdown);
        Html::VRef(div.into())
    }
}
