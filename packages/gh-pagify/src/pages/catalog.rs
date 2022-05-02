use crate::api::get_file_tree;
use crate::entities::MapAssets;
use yew::prelude::*;

// Definition
pub struct Catalog {
    map_assets: MapAssets,
}

// Props
#[derive(PartialEq, Properties)]
pub struct CatalogProps;

// State
pub enum CatalogMsg {
    SetAssets(MapAssets),
}

// Implementation
impl Component for Catalog {
    type Message = CatalogMsg;
    type Properties = CatalogProps;

    // On Initialize
    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async move {
            let body = get_file_tree().await;
            CatalogMsg::SetAssets(MapAssets::from(body))
        });

        Self {
            map_assets: MapAssets { assets: vec![] },
        }
    }

    // When state change
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CatalogMsg::SetAssets(assets) => {
                self.map_assets = assets;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            }
        }
    }

    // On Render
    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div>{"hello"}</div>
        }
    }
}
