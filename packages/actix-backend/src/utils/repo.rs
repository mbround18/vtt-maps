use crate::utils::folders::{assets_dir, thumbnails_dir};
use anyhow::{Context, Result};
use git2::{
    BranchType, FetchOptions, RemoteCallbacks, Repository, ResetType,
    build::{CheckoutBuilder, RepoBuilder},
};
use shared::utils::root_dir::root_dir;
use std::{env, fs};
use tracing::{debug, error, info};

pub fn update_repo() -> Result<()> {
    // 1) figure out where to put the repo
    let root = root_dir().context("Failed to resolve root directory")?;
    debug!("Using repo path: {}", root.display());

    // 2) ensure the directory exists
    fs::create_dir_all(&root).with_context(|| format!("Failed to create `{}`", root.display()))?;

    // 3) branch & URL
    let branch = env::var("REPO_REF").unwrap_or_else(|_| "main".into());
    let url = "https://github.com/dnd-apps/vtt-maps.git";

    // 4) decide clone vs. update by checking for `.git`
    let is_git_repo = root.join(".git").exists();
    if !is_git_repo {
        info!("No .git found—cloning {}@{}…", url, branch);

        let cb = RemoteCallbacks::new();
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);

        RepoBuilder::new()
            .branch(&branch)
            .fetch_options(fo)
            .clone(url, &root)
            .with_context(|| format!("Failed to clone {} → {}", url, root.display()))?;
    } else {
        info!(".git detected—opening existing repo at {}", root.display());

        let repo = Repository::open(&root)
            .with_context(|| format!("Failed to open repo at `{}`", root.display()))?;

        let cb = RemoteCallbacks::new();
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);

        let mut remote = repo
            .find_remote("origin")
            .or_else(|_| repo.remote_anonymous("origin"))
            .context("Could not find `origin` remote")?;
        remote
            .fetch(&[&branch], Some(&mut fo), None)
            .with_context(|| format!("Failed to fetch branch `{}`", branch))?;

        let fetch_ref = format!("refs/remotes/origin/{}", branch);
        let fetch_commit = repo
            .find_reference(&fetch_ref)
            .with_context(|| format!("Reference `{}` not found", fetch_ref))?
            .peel_to_commit()
            .context("Failed to peel fetched reference to commit")?;

        // ensure local branch exists
        if repo.find_branch(&branch, BranchType::Local).is_err() {
            repo.branch(&branch, &fetch_commit, true)
                .with_context(|| format!("Failed to create local branch `{}`", branch))?;
        }

        // hard‐reset working tree
        repo.set_head(&format!("refs/heads/{}", branch))
            .with_context(|| format!("Failed to set HEAD to `{}`", branch))?;
        repo.checkout_head(Some(CheckoutBuilder::default().force()))
            .context("Checkout failed")?;
        repo.reset(fetch_commit.as_object(), ResetType::Hard, None)
            .context("Hard reset to fetched commit failed")?;
    }

    // 5) initialize your asset/thumb folders
    let assets = assets_dir().context("Failed to initialize assets directory")?;
    info!("Assets dir ready: {}", assets.display());

    let thumbs = thumbnails_dir().context("Failed to initialize thumbnails directory")?;
    info!("Thumbnails dir ready: {}", thumbs.display());

    Ok(())
}

pub fn get_sha() -> Result<String, anyhow::Error> {
    let base = root_dir().unwrap_or_else(|_| {
        error!("Failed to resolve root directory");
        std::path::PathBuf::from(".")
    });

    match Repository::open(&base) {
        Ok(repo) => match repo.head().and_then(|head| head.peel_to_commit()) {
            Ok(commit) => Ok(commit.id().to_string()),
            Err(e) => {
                error!("Failed to read commit: {}", e);
                Err(anyhow::anyhow!("Failed to read commit: {}", e))
            }
        },
        Err(e) => {
            error!("Failed to open git repo: {}", e);
            Err(anyhow::anyhow!("Failed to open git repo: {}", e))
        }
    }
}
