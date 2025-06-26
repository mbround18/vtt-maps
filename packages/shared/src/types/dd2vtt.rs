use image::ImageReader;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use crate::decode;
use crate::types::map_resolution::MapResolution;
use serde_json;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DD2VTTFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    pub image: String,
    pub(crate) resolution: MapResolution,
}

impl DD2VTTFile {
    /// Creates a DD2VTT file from a file path.
    ///
    /// # Panics
    /// Panics if the file cannot be opened, read, or parsed as JSON.
    #[must_use]
    pub fn from_path(value: PathBuf) -> Self {
        let mut file = fs::File::open(&value).expect("Unable to open file");
        let mut data = String::new();
        file.read_to_string(&mut data).expect("Unable to read file");
        let mut dd2vtt_file: DD2VTTFile = serde_json::from_str(&data).expect("Unable to parse");
        dd2vtt_file.path = Some(value);
        dd2vtt_file
    }

    /// Exports a thumbnail image to the specified output path.
    ///
    /// # Panics
    /// Panics if the image format cannot be guessed, the image cannot be decoded, or the thumbnail cannot be saved.
    pub fn export_thumbnail_file(self, output: &Path) {
        let bytes = decode(self.image);
        let img = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .expect("Unable to guess image format")
            .decode()
            .expect("Unable to decode image");
        let thumbnail = img.thumbnail(img.width() / 16, img.height() / 16);
        thumbnail.save(output).expect("Unable to save thumbnail");
    }
}
