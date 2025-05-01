use crate::types::map_resolution::MapResolution;
use crate::utils::root_dir::root_dir;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
        let data = std::fs::read_to_string(value)
            .unwrap_or_else(|_| panic!("Unable to read file: {:?}", value.to_str()));
        let mut content: Self = serde_json::from_str(&data).unwrap_or_else(|e| {
            panic!(
                "Unable to parse JSON: {:?},\nerror: {:?}",
                value.to_str(),
                e
            )
        });
        if content.path.starts_with("maps") {
            let relative_path = content.path;
            content.path = root_dir().map_or(relative_path.to_string(), |mut root| {
                root.push(relative_path);
                root.to_str().unwrap_or("").to_string()
            });
        }
        content
    }
}

use crate::types::dd2vtt::DD2VTTFile;

impl From<DD2VTTFile> for MapReference {
    fn from(value: DD2VTTFile) -> Self {
        let path = value.path.expect("DD2VTTFile path missing");
        let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|_| panic!("Unable to read file bytes: {:?}", path));
        let hash = Sha256::digest(&bytes);

        MapReference {
            name: file_name.replace(".dd2vtt", ""),
            path: path.to_string_lossy().to_string(),
            hash: format!("{:x}", hash),
            bytes: bytes.len() as u64,
            resolution: value.resolution,
        }
    }
}

impl MapReference {
    pub fn to_file(&self, output: &PathBuf) {
        println!("Writing file info {:?}", &self);
        let data = serde_json::to_string_pretty(&self).expect("Unable to serialize");
        std::fs::write(output, data).expect("Unable to write file");
    }
}
