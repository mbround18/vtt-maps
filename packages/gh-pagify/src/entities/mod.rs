use crate::api::gh::{GHRepoTree, GitTree};

pub struct MapAsset {
    pub download_url: String,
    pub preview_url: String,
}

impl From<&GitTree> for MapAsset {
    fn from(tree: &GitTree) -> Self {
        let preview_url = format!(
            "https://raw.githubusercontent.com/dnd-apps/vtt-maps/main/{}",
            &tree.path.replace(" ", "%20")
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

pub struct MapAssets {
    pub(crate) assets: Vec<MapAsset>,
}

impl From<GHRepoTree> for MapAssets {
    fn from(repo: GHRepoTree) -> Self {
        let assets = repo
            .tree
            .iter()
            .filter(|e| e.path.contains(".preview.png"))
            .map(MapAsset::from)
            .collect();
        MapAssets { assets }
    }
}
