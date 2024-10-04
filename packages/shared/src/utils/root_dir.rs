use std::{env, io, path::PathBuf};

pub fn root_dir() -> io::Result<PathBuf> {
    let dir = env::current_dir()?;
    let canonical_path = dir.canonicalize()?;
    Ok(canonical_path)
}
