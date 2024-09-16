use image::ImageReader;
use serde::{Deserialize, Serialize};
use shared::types::map_reference::MapReference;
use shared::types::map_resolution::MapResolution;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DD2VTTFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) path: Option<String>,
    pub(crate) image: String,
    pub(crate) resolution: MapResolution,
}

impl From<String> for DD2VTTFile {
    fn from(value: String) -> DD2VTTFile {
        let data = fs::read_to_string(&value).expect("Unable to read file");
        let mut file: DD2VTTFile = serde_json::from_str(&data).expect("Unable to parse");
        file.path = Some(value);
        file
    }
}

impl From<DD2VTTFile> for MapReference {
    fn from(val: DD2VTTFile) -> Self {
        let file_path = val.path.expect("Path not found");
        let file_name = Path::new(&file_path).file_name().unwrap().to_str().unwrap();
        let bytes = fs::read(&file_path).unwrap();
        let hash = sha256::digest(&bytes);
        MapReference {
            name: file_name.replace(".dd2vtt", "").to_string(),
            path: String::from(&file_path[{ file_path.find("maps").unwrap() + 5 }..]),
            hash,
            bytes: bytes.len() as u64,
            resolution: val.resolution,
        }
    }
}

impl DD2VTTFile {
    pub fn to_thumbnail_file(&self, output: PathBuf) {
        println!("Generating thumbnail for {:?}", output);

        let bytes = shared::decode(String::from(&self.image));
        let img2 = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
        let thumbnail = img2.thumbnail(img2.width() / 16, img2.height() / 16);
        thumbnail.save(&output).unwrap();
    }
}
