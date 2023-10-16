use crate::types::map_resolution::MapResolution;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct MapReference {
    pub name: String,
    pub path: String,
    pub hash: String,
    pub bytes: u64,
    pub resolution: MapResolution,
}

impl From<&PathBuf> for MapReference {
    fn from(value: &PathBuf) -> Self {
        let data = std::fs::read_to_string(value).expect("Unable to read file");
        serde_json::from_str(&data).expect("Unable to parse")
    }
}

impl MapReference {
    pub fn to_file(&self, output: &PathBuf) {
        println!("Writing file info {:?}", &self);
        let data = serde_json::to_string_pretty(&self).expect("Unable to serialize");
        std::fs::write(output, data).expect("Unable to write file");
    }
}
