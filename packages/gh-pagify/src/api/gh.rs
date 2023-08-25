use reqwasm::http::Request;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize)]
pub struct GitTree {
    pub path: String,
}

impl From<String> for GitTree {
    fn from(path: String) -> Self {
        GitTree { path }
    }
}

impl Clone for GitTree {
    fn clone(&self) -> Self {
        Self {
            path: self.path.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GHRepoTree {
    pub tree: Vec<GitTree>,
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

// pub async fn get_body_as_string(url: String) -> DD2VTT {
//     Request::get(&url)
//         .send()
//         .await
//         .unwrap()
//         .json()
//         .await
//         .unwrap()
// }
