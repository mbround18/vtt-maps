use image::{ ImageReader};
use serde::{Deserialize, Serialize};
use shared::types::{map_reference::MapReference, map_resolution::MapResolution};
use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::io::AsyncReadExt;
use std::path::{Path, PathBuf};
use std::io::Cursor;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DD2VTTFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) path: Option<PathBuf>,
    pub(crate) image: String,
    pub(crate) resolution: MapResolution,
}

impl DD2VTTFile {
    pub async fn from_path(value: PathBuf) -> Self {
        let mut file = fs::File::open(&value).await.expect("Unable to open file");
        let mut data = String::new();
        file.read_to_string(&mut data).await.expect("Unable to read file");
        let mut dd2vtt_file: DD2VTTFile = serde_json::from_str(&data).expect("Unable to parse");
        dd2vtt_file.path = Some(value);
        dd2vtt_file
    }

    pub async fn to_map_reference(&self) -> MapReference {
        let file_path = self.path.clone().expect("Path not found");
        let file_name = file_path.file_name().unwrap().to_string_lossy().into_owned();
        let bytes = fs::read(&file_path).await.expect("Unable to read file bytes");
        let hash = Sha256::digest(&bytes);
        MapReference {
            name: file_name.replace(".dd2vtt", ""),
            path: file_path.to_string_lossy().to_string(),
            hash: format!("{:x}", hash),
            bytes: bytes.len() as u64,
            resolution: self.resolution.clone(),
        }
    }

    pub async fn to_thumbnail_file(self, output: &Path) {
        let bytes = shared::decode(self.image);
        let img = ImageReader::new(Cursor::new(bytes))
          .with_guessed_format()
          .expect("Unable to guess image format")
          .decode()
          .expect("Unable to decode image");
        let thumbnail = img.thumbnail(img.width() / 16, img.height() / 16);
        thumbnail.save(output).expect("Unable to save thumbnail");
    }
}
