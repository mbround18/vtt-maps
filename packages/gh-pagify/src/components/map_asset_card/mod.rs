use crate::entities::MapAsset;
use yew::prelude::*;

// Definition
pub struct MapAssetCard {
    asset: MapAsset,
}

// Props
#[derive(PartialEq, Properties)]
pub struct MapAssetCardProps {
    pub asset: MapAsset,
}

// Implementation
impl Component for MapAssetCard {
    type Message = ();
    type Properties = MapAssetCardProps;

    // On Initialize
    fn create(ctx: &Context<Self>) -> Self {
        let props = &ctx.props();
        let asset = props.asset.to_owned();
        Self { asset }
    }

    // On Render
    fn view(&self, _ctx: &Context<Self>) -> Html {
        let src = (&self.asset.preview_url).to_string();
        let name = (&self.asset.name).to_string();
        let download_url = (&self.asset.download_url).to_string();

        // let identifier =
        //     urlencoding::encode({ &base64::encode(&self.asset.tree.path) }).to_string();

        // let href = format!("/catalog/{}", identifier);

        html! {
            <div class={"card"}>
                <h3>{format!("{}", name)}</h3>
                <img {src} class={"preview-image"} />
                <a target={"_blank"} href={download_url}>{
                    "Download DD2VTT File"
                }</a>
                // <a {href}>{format!("View {}", name)}</a>
                // <p>{identifier}</p>
            </div>
        }
    }
}
