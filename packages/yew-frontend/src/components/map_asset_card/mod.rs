// use crate::api::api::ApiEndpoint;
use shared::types::map_document::MapDocument;
use shared::utils::casing::titlecase;
use yew::prelude::*;

// Definition
pub struct MapAssetCard {
    asset: MapDocument,
}

// Props
#[derive(PartialEq, Properties)]
pub struct MapAssetCardProps {
    pub asset: MapDocument,
}

// Implementation
impl Component for MapAssetCard {
    type Message = ();
    type Properties = MapAssetCardProps;

    fn create(ctx: &Context<Self>) -> Self {
        let props = &ctx.props();
        let asset = props.asset.clone();
        Self { asset }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let src = self.asset.thumbnail.to_string();
        let name = String::from(
            &self
                .asset
                .name
                .clone()
                .split('-')
                .flat_map(|e| e.split('_'))
                .map(titlecase)
                .collect::<Vec<String>>()
                .join(" "),
        );
        // let download_url = ApiEndpoint::DownloadMap {
        //     id: self.asset.id.clone(),
        // }
        // .url();

        html! {
            <div class={"card map-asset"}>
                <h3>{name.to_string()}</h3>
                <img {src} class={"preview-image"} />
                <div class={"card-actions"}>
                    // <a
                    //     href={download_url.clone()}
                    //     download={"true"}
                    //     class="btn btn-primary"
                    // >
                    //     { "Download DD2VTT File" }
                    // </a>
                    <a
                        href={format!("/maps/{}", self.asset.id)}
                        class="btn btn-primary width-100"
                    >
                        { "Explore" }
                    </a>
                </div>
            </div>
        }
    }
}
