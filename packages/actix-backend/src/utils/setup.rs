use crate::utils::folders::{assets_dir, thumbnails_dir};
use anyhow::Result;
use git2::Repository;
use shared::utils::root_dir::root_dir;
use std::fs;

pub fn setup_folders() -> Result<()> {
    // 1) Ensure root exists (clone if not)
    let root = root_dir()?;
    if std::env::var("REPO_DIR").is_ok() {
        if !root.exists() {
            Repository::clone("https://github.com/dnd-apps/vtt-maps", &root)?;
        }
        let assets = assets_dir()?;
        if !assets.exists() {
            fs::create_dir_all(&assets)?;
        }
        let thumbs = thumbnails_dir()?;
        if !thumbs.exists() {
            fs::create_dir_all(&thumbs)?;
        }
    }

    Ok(())
}
