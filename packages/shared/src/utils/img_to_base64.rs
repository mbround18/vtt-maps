use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use image::ImageOutputFormat;

pub fn image_to_base64(file: &PathBuf) -> String {
    let mut image_data = fs::read(file).unwrap();
    let image = image::load_from_memory(&image_data).unwrap();

    image.write_to(&mut Cursor::new(&mut image_data), ImageOutputFormat::Png)
        .unwrap();
    const CUSTOM_ENGINE: engine::GeneralPurpose =
        engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

    CUSTOM_ENGINE.encode(image_data)
}
