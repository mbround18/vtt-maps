use crate::types::map_resolution::MapResolution;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MapDocument {
    pub id: String,
    pub name: String,
    pub path: String,
    pub thumbnail: String,
    pub content: Option<String>,
    pub resolution: MapResolution,
}
