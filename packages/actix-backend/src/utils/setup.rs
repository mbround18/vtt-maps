use crate::utils::folders::{assets_dir, thumbnails_dir};
use anyhow::{Context, Result};
use git2::{FetchOptions, build::RepoBuilder};
use shared::utils::root_dir::root_dir;
use tracing::{debug, info};

pub fn setup_folders() -> Result<()> {
    let root = root_dir()?;
    let repo_ref = std::env::var("REPO_REF").unwrap_or_else(|_| "main".to_string());

    debug!("Resolved root directory: {}", root.display());
    let has_repo_dir = std::env::var("REPO_DIR").is_ok();
    let is_git_repo = root.join(".git").exists();
    let is_not_empty = root
        .read_dir()
        .map(|mut dir| dir.next().is_some())
        .unwrap_or(false);

    if has_repo_dir && !is_git_repo && is_not_empty {
        debug!("Cleaning up the directory: {}", root.display());
        for entry in root.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                std::fs::remove_dir_all(&path)?;
            } else {
                std::fs::remove_file(&path)?;
            }
        }
    } else if has_repo_dir && is_git_repo {
        debug!("Updating the repository: {}", root.display());
        let repo = git2::Repository::open(&root)?;
        let mut remote = repo.find_remote("origin")?;
        remote.fetch(&[&repo_ref], None, None)?;
    }

    if has_repo_dir && !is_git_repo {
        let repo_url = "https://github.com/dnd-apps/vtt-maps";
        let branch = std::env::var("REPO_REF").unwrap_or_else(|_| repo_ref.clone());

        info!(
            "Cloning branch '{}' from {} into {}",
            branch,
            repo_url,
            root.display()
        );

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.download_tags(git2::AutotagOption::All);

        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_opts).branch(&branch);

        builder
            .clone(repo_url, &root)
            .with_context(|| format!("Failed to clone branch '{branch}' from {repo_url}"))?;

        info!("Clone complete.");
    } else {
        info!("Skipping clone — .git directory already exists or REPO_DIR is not set.");
    }

    let assets = assets_dir().context("Failed to initialize assets directory")?;
    info!("Assets directory ready: {}", assets.display());

    let thumbs = thumbnails_dir().context("Failed to initialize thumbnails directory")?;
    info!("Thumbnails directory ready: {}", thumbs.display());

    Ok(())
}
