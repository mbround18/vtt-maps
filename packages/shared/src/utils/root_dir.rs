use std::{env, fs, io, path::PathBuf};

pub fn root_dir() -> io::Result<PathBuf> {
    let dir = match env::var("REPO_PATH") {
        Ok(path) => PathBuf::from(path),
        Err(_) => env::current_dir()?,
    };
    fs::canonicalize(dir)
}

pub fn maps_dir() -> io::Result<PathBuf> {
    let mut path = root_dir().unwrap_or_else(|e| panic!("Unable to get root directory: {:?}", e));
    path.push("maps");
    fs::canonicalize(path)
}
