mod api;
mod entities;

use crate::api::get_file_tree;
use crate::entities::MapAssets;
use yew::prelude::*;
use crate::api::local::get_readme;

enum Msg {
    SetReadme(String),
    SetAssets(MapAssets),
}

struct Model {
    readme: String,
    map_assets: MapAssets,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async move {
            let body = get_file_tree().await;
            Msg::SetAssets(MapAssets::from(body))
        });
        ctx.link().send_future(async move {
            let body = get_readme().await;
            Msg::SetReadme(body)
        });
        Self {
            readme: String::from("# VTT-Maps"),
            map_assets: MapAssets { assets: vec![] },
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetAssets(assets) => {
                self.map_assets = assets;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            },
            Msg::SetReadme(readme) => {
                self.readme = markdown::to_html(&readme);
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let assets = &self.map_assets.assets;
        let items = assets
            .into_iter()
            .map(|e| {
                let href = format!("{}", e.download_url);
                let src = format!("{}", e.preview_url);

                html! {
                    <div class="card" key={src.clone()}>
                        <img alt={src.clone()} src={src.clone()}  />
                        <div class="overlay">
                            <a {href} target="_blank">{"^ Download Above ^"}</a>
                        </div>
                    </div>
                }
            })
            .collect::<Vec<_>>();

        let div = gloo_utils::document().create_element("div").unwrap();
        div.set_inner_html(&self.readme);

        html! {
            <>
            { Html::VRef(div.into()) }
            <h2>{"Catalogue"}</h2>
            <hr />
            <div class="image-mosaic">{
                for items
            }</div>
            </>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
