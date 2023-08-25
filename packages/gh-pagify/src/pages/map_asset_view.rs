use crate::api::gh::GitTree;
use crate::entities::MapAsset;
use yew::prelude::*;

// Definition
pub struct MapAssetView {
    asset: MapAsset,
    loaded: bool,
    image: String,
    dd2vtt: String,
}

impl MapAssetView {
    fn parse_path(id: &str) -> String {
        String::from_utf8({
            base64::decode(urlencoding::decode(id).expect("UTF-8").to_string()).unwrap()
        })
        .unwrap()
    }
    fn load_dd2vtt(ctx: &Context<MapAssetView>) {
        let path: String = MapAssetView::parse_path(&ctx.props().id);
        let _dd2vtt_url = path.clone().replace(".preview.png", "dd2vtt");

        // ctx.link().callback_future_once(async move {
        //
        //     dd2vtt_url
        // });
    }
}

// Props
#[derive(PartialEq, Properties)]
pub struct MapAssetViewProps {
    pub id: String,
}

// State
pub enum MapAssetViewMsg {
    SetLoaded(bool),
}

// Implementation
impl Component for MapAssetView {
    type Message = MapAssetViewMsg;
    type Properties = MapAssetViewProps;

    // On Initialize
    fn create(ctx: &Context<Self>) -> Self {
        let path: String = MapAssetView::parse_path(&ctx.props().id);
        let asset = MapAsset::from(&GitTree::from(path));
        let image = asset.preview_url.clone();
        Self {
            asset,
            loaded: false,
            image,
            dd2vtt: "".to_string(),
        }
    }

    // When state change
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            MapAssetViewMsg::SetLoaded(b) => {
                self.loaded = b;
                true
            }
        }
    }

    // On Render
    fn view(&self, _ctx: &Context<Self>) -> Html {
        let name = (&self.asset.name).to_string();
        let src = (&self.image).to_string();
        let download_url = (&self.asset.download_url).to_string();
        html! {
            <div id={"map-asset-view"} class="card">
                <h1>{name}</h1>
                <img {src} />
                <a target={"_blank"} href={download_url}>{"Download DD2VTT File"}</a>
            </div>
        }
    }
}
