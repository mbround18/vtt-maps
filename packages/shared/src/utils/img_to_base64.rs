use base64::{engine::general_purpose, Engine as _};
use image::ImageOutputFormat;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

pub fn image_to_base64(file: &PathBuf) -> String {
    let mut image_data = fs::read(file).unwrap();
    let image = image::load_from_memory(&image_data).unwrap();

    image
        .write_to(&mut Cursor::new(&mut image_data), ImageOutputFormat::Png)
        .unwrap();

    general_purpose::STANDARD.encode(image_data)
}
