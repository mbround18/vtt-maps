use base64::{Engine as _, engine::general_purpose};
use image::{ImageError, ImageFormat};
use std::io::Cursor;
use std::{fs, io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImageConversionError {
    #[error("Failed to read the file: {0}")]
    FileReadError(#[source] io::Error),

    #[error("Failed to load image from memory: {0}")]
    ImageLoadError(#[source] ImageError),

    #[error("Failed to write image to memory: {0}")]
    ImageWriteError(#[source] ImageError),
}

impl From<io::Error> for ImageConversionError {
    fn from(error: io::Error) -> Self {
        ImageConversionError::FileReadError(error)
    }
}

impl From<ImageError> for ImageConversionError {
    fn from(error: ImageError) -> Self {
        ImageConversionError::ImageLoadError(error)
    }
}

pub fn image_to_base64(file: &PathBuf) -> Result<String, ImageConversionError> {
    let mut image_data = fs::read(file)?;
    let image = image::load_from_memory(&image_data)?;

    image.write_to(&mut Cursor::new(&mut image_data), ImageFormat::Png)?;

    Ok(general_purpose::STANDARD.encode(image_data))
}
