use shared::utils::root_dir::root_dir;
use std::fs::{canonicalize, create_dir_all};
use std::io;
use std::path::PathBuf;

pub fn assets_dir() -> Result<PathBuf, io::Error> {
    let mut path = root_dir()?;
    path.push("assets");
    create_dir_all(&path)?;
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Assets path missing: {}", path.display()),
        ));
    }
    canonicalize(path)
}

pub fn thumbnails_dir() -> Result<PathBuf, io::Error> {
    let mut path = assets_dir()?;
    path.push("thumbnails");
    create_dir_all(&path)?;
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Thumbnails path missing: {}", path.display()),
        ));
    }
    canonicalize(path)
}
