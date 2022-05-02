use crate::entities::MapAsset;
use yew::prelude::*;

// Definition
pub struct MapAssetCard {
    asset: MapAsset,
}

// Props
#[derive(PartialEq, Properties)]
pub struct MapAssetCardProps {
    asset: MapAsset,
}

// Implementation
impl Component for MapAssetCard {
    type Message = ();
    type Properties = MapAssetCardProps;

    // On Initialize
    fn create(ctx: &Context<Self>) -> Self {
        let props = &ctx.props();
        let asset = props.asset.clone();
        Self { asset }
    }

    // On Render
    fn view(&self, _ctx: &Context<Self>) -> Html {
        let download_url = &self.asset.download_url;
        html! {
            <div>{download_url}</div>
        }
    }
}
