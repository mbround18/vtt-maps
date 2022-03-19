use crate::entities::assets::Asset;
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use gloo_console::console;

#[derive(Serialize, Deserialize)]
pub struct GitTree {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct GHRepoTree {
    pub tree: Vec<GitTree>,
}

impl From<GHRepoTree> for HashMap<String, Asset> {
    fn from(repo: GHRepoTree) -> Self {
        let mut assets: HashMap<String, Asset> = HashMap::new();

        for i in repo.tree {
            let path = i.path;

            // Remove generic things
            if (! path.contains('.')) || path.contains("README.md") {
                continue;
            }

            // Skip stuff we do not care about.
            if ! [".preview.png", ".dd2vtt", ".md", "README.md"]
                .iter()
                .any(|e| path.contains(e))
            {
                console!(format!("Is not match: {}", path));
                continue;
            }

            let boolean_list: &mut HashMap<String, bool> = &mut HashMap::new();

            // Parse Tags
            let mut tags = path.split('/').map(String::from).collect::<Vec<String>>();
            let asset_tag = vec![String::from("asset")];
            let ra = (tags.len() - 1)..(tags.len());

            console!(String::from(tags.join("|")));

            let file_name = {
                let parts = tags.splice(ra, asset_tag).collect::<Vec<String>>();
                String::from(parts.first().unwrap())
            };

            let base_parts = file_name.split('.').collect::<Vec<&str>>();
            let base_name = String::from((base_parts.get(0).unwrap_or(&"unknown")).to_owned());
            // Handle Assignment

            if file_name.ends_with(".preview.png") {
            } else if file_name.ends_with(".dd2vtt") {
                boolean_list.insert(String::from("has_vtt_file"), true);
            } else if file_name.ends_with(".md") {
                boolean_list.insert(String::from("has_details"), true);
            }

            assets.insert(base_name, Asset::new(path, tags, boolean_list));
        }

        assets
    }
}

pub async fn get_file_tree() -> GHRepoTree {
    Request::get("https://api.github.com/repos/dnd-apps/vtt-maps/git/trees/main?recursive=1")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}
