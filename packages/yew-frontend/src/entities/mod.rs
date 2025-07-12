mod dd2vtt;

use crate::api::gh::{GHRepoTree, GitTree};
use gloo_console::debug;
use shared::utils::casing::titlecase;

#[derive(PartialEq)]
pub struct MapAsset {
    pub tree: GitTree,
    pub name: String,
    pub download_url: String,
    pub preview_url: String,
}

impl From<&GitTree> for MapAsset {
    fn from(tree: &GitTree) -> Self {
        let name = tree
            .path
            .split('/')
            .next_back()
            .unwrap()
            .replace(".preview.png", "")
            .split('-')
            .map(titlecase)
            .collect::<Vec<String>>()
            .join(" ");
        let preview_url = format!(
            "https://raw.githubusercontent.com/dnd-apps/vtt-maps/main/{}",
            &tree.path.replace(' ', "%20")
        );
        let download_url = format!(
            // "https://raw.githubusercontent.com/dnd-apps/vtt-maps/main/{}",
            "https://minhaskamal.github.io/DownGit/#/home?url=https://github.com/dnd-apps/vtt-maps/blob/main/{}",
            tree.path
                .clone()
                .replace(".preview.png", ".dd2vtt")
                .replace("..", ".")
        );
        MapAsset {
            tree: tree.clone(),
            name,
            download_url,
            preview_url,
        }
    }
}

impl Clone for MapAsset {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree.clone(),
            name: self.name.to_string(),
            download_url: self.download_url.to_string(),
            preview_url: self.preview_url.to_string(),
        }
    }
}

pub struct MapAssets {
    // pub(crate) assets: Vec<MapAsset>,
}

impl From<GHRepoTree> for MapAssets {
    fn from(repo: GHRepoTree) -> Self {
        for x in repo.tree {
            debug!("{}", x.path);
        }
        // let assets = repo
        //     .tree
        //     .iter()
        //     .filter(|e| e.path.contains(".preview.png"))
        //     .map(MapAsset::from)
        //     .collect();
        MapAssets {}
    }
}
