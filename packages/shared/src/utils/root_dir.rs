use std::{env, fs, io, path::PathBuf};

pub fn root_dir() -> io::Result<PathBuf> {
    match env::var("REPO_DIR") {
        Ok(path) => {
            let path = PathBuf::from(path);
            if !path.exists() {
                fs::create_dir_all(&path).map_err(|e| {
                    eprintln!("Failed to create REPO_DIR {}: {}", path.display(), e);
                    e
                })?;
            }
            Ok(path)
        }
        Err(_) => {
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
}

pub fn maps_dir() -> io::Result<PathBuf> {
    let mut path = root_dir()?;
    path.push("maps");

    // Create maps directory if it doesn't exist
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    fs::canonicalize(path)
}
