use glob::{glob_with, Paths, PatternError};
use std::fs;
use std::path::{Path, PathBuf};

pub fn path_to_thumbnail_path(file_path: &str) -> PathBuf {
    Path::new(file_path)
        .with_extension("")
        .with_extension("preview.png")
}

pub fn get_files(base_directory: &Path) -> Result<Paths, PatternError> {
    let src_path = fs::canonicalize(base_directory).unwrap();
    let glob_pattern = Path::new(&src_path).join("**").join("*.dd2vtt");
    glob_with(glob_pattern.to_str().unwrap(), Default::default())
}
