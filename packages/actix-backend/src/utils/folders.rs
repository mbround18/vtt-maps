use shared::utils::root_dir::root_dir;
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn assets_dir() -> Result<PathBuf, io::Error> {
    let mut path = root_dir().unwrap_or_else(|e| panic!("Unable to get root directory: {:?}", e));
    path.push("assets");
    fs::canonicalize(path)
}

pub fn thumbnails_dir() -> Result<PathBuf, io::Error> {
    let mut path =
        assets_dir().unwrap_or_else(|e| panic!("Unable to get assets directory: {:?}", e));
    path.push("thumbnails");
    fs::create_dir_all(&path)?;
    fs::canonicalize(path)
}
