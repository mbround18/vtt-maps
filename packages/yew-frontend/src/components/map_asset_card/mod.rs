// use crate::api::api::ApiEndpoint;
use shared::types::map_document::MapDocument;
use shared::utils::casing::titlecase;
use yew::prelude::*;

// Definition
pub struct MapAssetCard {
    asset: MapDocument,
    download_count: Option<i64>,
    download_last_30d: Option<i64>,
}

// Props
#[derive(PartialEq, Properties)]
pub struct MapAssetCardProps {
    pub asset: MapDocument,
    #[prop_or(None)]
    pub download_count: Option<i64>,
    #[prop_or(None)]
    pub download_last_30d: Option<i64>,
}

// Implementation
impl Component for MapAssetCard {
    type Message = ();
    type Properties = MapAssetCardProps;

    fn create(ctx: &Context<Self>) -> Self {
        let props = &ctx.props();
        let asset = props.asset.clone();
        Self {
            asset,
            download_count: props.download_count,
            download_last_30d: props.download_last_30d,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        self.asset = props.asset.clone();
        self.download_count = props.download_count;
        self.download_last_30d = props.download_last_30d;
        true
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
                {
                    if let Some(count) = self.download_count {
                        let label = if count == 1 { "download" } else { "downloads" };
                        let count_text = count.to_string();
                        let last_30d = self.download_last_30d.unwrap_or(0);
                        html! {
                            <div class="download-stat" aria-label={format!("{count_text} {label}")}>
                                <span class="download-count">{count_text}</span>
                                <span class="download-label">{label}</span>
                                {
                                    if last_30d > 0 {
                                        html! {
                                            <span class="download-trend">{format!("{last_30d} last 30d")}</span>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
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
