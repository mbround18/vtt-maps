use crate::api::get_file_tree;
use crate::entities::assets::Asset;
use crate::entities::MapAsset;
use std::collections::HashMap;
use yew::prelude::*;

pub enum Msg {
    SetAssets(HashMap<String, Asset>),
    SetTags(HashMap<String, bool>),
    UpdateTag(String, bool)
}

pub struct Catalogue {
    assets: HashMap<String, Asset>,
    tags: HashMap<String, bool>
}

impl Catalogue {
    fn render_tags(self, tags: Vec<String>) -> Html {
        html! {
            <ul>
            {
                tags.into_iter().map(|name| {
                    let display = name.clone();
                    let new_state = !self.tags.get(&name).unwrap_or(&false).to_owned();
                    html!{
                        <li key={name.clone()} onclick={Callback::from(move |_| {
                            self::Msg::UpdateTag(String::from(&name), new_state);
                        })}>
                            <div>{ format!("{}!", display) }</div>
                        </li>
                    }
                }).collect::<Html>()
            }
            </ul>
        }
    }
}

impl Component for Catalogue {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async move {
            let body = get_file_tree().await;
            Msg::SetAssets(HashMap::from(body))
        });
        Self {
            assets: HashMap::new(),
            tags: HashMap::new()
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetAssets(assets) => {
                self.assets = assets;

                let mut tags: HashMap<String, bool> = HashMap::new();
                for asset in self.assets.values() {
                    for tag in &asset.tags {
                        tags.insert(String::from(tag), false);
                    }
                }
                Msg::SetTags(tags);

                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            },
            Msg::SetTags(tags) => {
                self.tags = tags;
                true
            },
            Msg::UpdateTag(tag, value) => {
                self.tags.insert(tag, value);
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let items = self
            .assets
            .iter()
            .map(|(name, asset)| {
                let map_assert = MapAsset::from(asset);
                let href = map_assert.download_url;
                let src = map_assert.preview_url;

                html! {
                    <div class="card" key={src.clone()}>
                        <h2>{name}</h2>
                        <img alt={src.clone()} src={src.clone()}  />
                        <div class="overlay">
                            <a {href} target="_blank">{"^ Download Above ^"}</a>
                        </div>
                    </div>
                }
            })
            .collect::<Vec<_>>();

        let all_tags: Vec<String> = self.tags.keys().into_iter().map(String::from).collect();

        html! {
            <>
            <h2>{"Catalogue"}</h2>
            <div>
                {&self.render_tags(all_tags)}
            </div>
            <hr />
            <div class="image-mosaic">{
                for items
            }</div>
            </>
        }
    }
}
