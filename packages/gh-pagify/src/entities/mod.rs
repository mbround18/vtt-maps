pub mod assets;

use crate::api::gh::GitTree;
use crate::entities::assets::Asset;

pub struct MapAsset {
    pub download_url: String,
    pub preview_url: String,
}

impl From<&Asset> for MapAsset {
    fn from(asset: &Asset) -> Self {
        let preview_url = format!(
            "https://raw.githubusercontent.com/dnd-apps/vtt-maps/main/{}",
            &asset.path.replace(' ', "%20")
        );
        let download_url = format!(
            "https://downgit.github.io/#/home?url=https://github.com/dnd-apps/vtt-maps/blob/main/{}",
            asset.path.clone().replace(".preview.png", ".dd2vtt").replace("..", ".")
        );
        MapAsset {
            download_url,
            preview_url,
        }
    }
}

impl From<&GitTree> for MapAsset {
    fn from(tree: &GitTree) -> Self {
        let preview_url = format!(
            "https://raw.githubusercontent.com/dnd-apps/vtt-maps/main/{}",
            &tree.path.replace(' ', "%20")
        );
        let download_url = format!(
            "https://downgit.github.io/#/home?url=https://github.com/dnd-apps/vtt-maps/blob/main/{}",
            tree.path.clone().replace(".preview.png", ".dd2vtt").replace("..", ".")
        );
        MapAsset {
            download_url,
            preview_url,
        }
    }
}
