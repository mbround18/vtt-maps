use std::{env, io, path::PathBuf};

pub fn root_dir() -> io::Result<PathBuf> {
    let mut dir = env::current_exe()?;
    dir.pop();

    if dir.ends_with("debug") || dir.ends_with("release") {
        dir.pop();
    }

    if dir.ends_with("target") {
        dir.pop();
    }

    Ok(dir)
}
