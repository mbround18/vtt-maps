use std::{env, fs, io, path::PathBuf};

/// Gets the root directory for the project.
///
/// # Errors
/// Returns an error if the root directory cannot be determined, created, or accessed.
pub fn root_dir() -> io::Result<PathBuf> {
    if let Ok(path) = env::var("REPO_DIR") {
        let path = PathBuf::from(path);
        if !path.exists() {
            fs::create_dir_all(&path).map_err(|e| {
                eprintln!("Failed to create REPO_DIR {}: {}", path.display(), e);
                e
            })?;
        }
        Ok(path)
    } else {
        let cwd = env::current_dir()?;
        if !cwd.join(".git").exists() && !cwd.join("Cargo.toml").exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Running locally, but could not validate project root at: {}",
                    cwd.display()
                ),
            ));
        }
        fs::canonicalize(cwd)
    }
}

/// Gets the maps directory within the project root.
///
/// # Errors
/// Returns an error if the root directory cannot be determined or the maps directory cannot be created.
pub fn maps_dir() -> io::Result<PathBuf> {
    let mut path = root_dir()?;
    path.push("maps");

    // Create maps directory if it doesn't exist
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    fs::canonicalize(path)
}
