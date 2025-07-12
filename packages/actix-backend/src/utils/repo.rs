use crate::utils::folders::{assets_dir, thumbnails_dir};
use anyhow::{Context, Result};
use git2::{
    BranchType, FetchOptions, Progress, RemoteCallbacks, Repository, ResetType,
    build::{CheckoutBuilder, RepoBuilder},
};
use shared::utils::root_dir::root_dir;
use std::{
    env, fs,
    sync::Arc,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};
use tracing::{debug, error, info, warn};

const DEFAULT_REPO_URL: &str = "https://github.com/dnd-apps/vtt-maps.git";
const DEFAULT_BRANCH: &str = "main";
const CLONE_PROGRESS_INTERVAL: usize = 100;
const FETCH_PROGRESS_INTERVAL: usize = 50;
const BYTES_TO_MB: usize = 1024 * 1024;

fn calculate_percentage(current: usize, total: usize) -> usize {
    if total > 0 {
        (100 * current) / total
    } else {
        0
    }
}

fn log_clone_progress(progress: &Progress) {
    let network_pct = calculate_percentage(progress.received_objects(), progress.total_objects());
    let index_pct = calculate_percentage(progress.indexed_objects(), progress.total_objects());
    let checkout_pct =
        calculate_percentage(progress.indexed_deltas(), progress.total_deltas().max(1));

    info!(
        "📦 Clone progress - Network: {}% ({}/{}), Index: {}% ({}/{}), Checkout: {}%",
        network_pct,
        progress.received_objects(),
        progress.total_objects(),
        index_pct,
        progress.indexed_objects(),
        progress.total_objects(),
        checkout_pct
    );

    if progress.received_bytes() > 0 {
        let mb_received = progress.received_bytes() / BYTES_TO_MB;
        debug!("📊 Downloaded {} MB", mb_received);
    }
}

fn log_commit_info(commit: &git2::Commit, prefix: &str) {
    info!("📋 {} commit: {}", prefix, commit.id());
    info!("👤 Author: {}", commit.author().name().unwrap_or("Unknown"));
    info!("💬 Message: {}", commit.summary().unwrap_or("No message"));
}

pub fn update_repo() -> Result<()> {
    let start_time = Instant::now();
    info!("🔄 Starting repository update process");

    // 1) figure out where to put the repo
    let root = root_dir().context("Failed to resolve root directory")?;
    info!("📁 Using repository path: {}", root.display());

    // 2) ensure the directory exists
    fs::create_dir_all(&root).with_context(|| format!("Failed to create `{}`", root.display()))?;

    // 3) branch & URL
    let branch = env::var("REPO_REF").unwrap_or_else(|_| DEFAULT_BRANCH.into());
    let url = DEFAULT_REPO_URL;
    info!("🌐 Repository URL: {}", url);
    info!("🌿 Target branch: {}", branch);

    // 4) decide clone vs. update by checking for `.git`
    let is_git_repo = root.join(".git").exists();
    if is_git_repo {
        info!(
            "🔍 .git detected—updating existing repo at {}",
            root.display()
        );
        update_existing_repository(&root, &branch)?;
    } else {
        info!("📥 No .git found—cloning {}@{}…", url, branch);
        clone_repository(url, &branch, &root)?;
    }

    // 5) initialize your asset/thumb folders
    let assets = assets_dir().context("Failed to initialize assets directory")?;
    info!("📂 Assets dir ready: {}", assets.display());

    let thumbs = thumbnails_dir().context("Failed to initialize thumbnails directory")?;
    info!("🖼️  Thumbnails dir ready: {}", thumbs.display());

    let elapsed = start_time.elapsed();
    info!("✅ Repository update completed in {:?}", elapsed);
    Ok(())
}

fn clone_repository(url: &str, branch: &str, root: &std::path::Path) -> Result<()> {
    let start_time = Instant::now();
    info!("🚀 Starting clone operation");

    let progress_counter = Arc::new(AtomicUsize::new(0));
    let progress_counter_clone = Arc::clone(&progress_counter);

    let mut cb = RemoteCallbacks::new();

    // Progress callback for clone
    cb.transfer_progress(move |progress: Progress| {
        let current = progress_counter_clone.fetch_add(1, Ordering::Relaxed);

        if current % CLONE_PROGRESS_INTERVAL == 0
            || progress.received_objects() == progress.total_objects()
        {
            log_clone_progress(&progress);
        }
        true
    });

    // Update callback
    cb.update_tips(|refname, old, new| {
        if old.is_zero() {
            info!("🔗 Created reference: {} -> {}", refname, new);
        } else {
            info!("🔄 Updated reference: {} {} -> {}", refname, old, new);
        }
        true
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);

    info!("🌐 Initiating clone from {}", url);
    let repo = RepoBuilder::new()
        .branch(branch)
        .fetch_options(fo)
        .clone(url, root)
        .with_context(|| format!("Failed to clone {} → {}", url, root.display()))?;

    let elapsed = start_time.elapsed();
    info!("✅ Clone completed successfully in {:?}", elapsed);

    // Verify clone
    let head_commit = repo.head()?.peel_to_commit()?;
    log_commit_info(&head_commit, "HEAD");

    Ok(())
}

fn update_existing_repository(root: &std::path::Path, branch: &str) -> Result<()> {
    let start_time = Instant::now();
    info!("🔄 Updating existing repository");

    let repo = Repository::open(root)
        .with_context(|| format!("Failed to open repo at `{}`", root.display()))?;

    // Get current commit before update
    let old_commit = repo
        .head()
        .and_then(|head| head.peel_to_commit())
        .map_or_else(|_| "unknown".to_string(), |commit| commit.id().to_string());
    info!("📋 Current commit: {}", old_commit);

    let progress_counter = Arc::new(AtomicUsize::new(0));
    let progress_counter_clone = Arc::clone(&progress_counter);

    let mut cb = RemoteCallbacks::new();

    // Progress callback for fetch
    cb.transfer_progress(move |progress: Progress| {
        let current = progress_counter_clone.fetch_add(1, Ordering::Relaxed);
        if (current % FETCH_PROGRESS_INTERVAL == 0
            || progress.received_objects() == progress.total_objects())
            && progress.total_objects() > 0
        {
            let pct = calculate_percentage(progress.received_objects(), progress.total_objects());
            info!(
                "📡 Fetch progress: {}% ({}/{} objects)",
                pct,
                progress.received_objects(),
                progress.total_objects()
            );
        }
        true
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);

    info!("🌐 Fetching from origin/{}", branch);
    let mut remote = repo
        .find_remote("origin")
        .or_else(|_| {
            warn!("⚠️  'origin' remote not found, creating anonymous remote");
            repo.remote_anonymous(DEFAULT_REPO_URL)
        })
        .context("Could not find or create remote")?;

    remote
        .fetch(&[branch], Some(&mut fo), None)
        .with_context(|| format!("Failed to fetch branch `{branch}`"))?;

    let fetch_ref = format!("refs/remotes/origin/{branch}");
    info!("🔍 Looking for reference: {}", fetch_ref);

    let fetch_commit = repo
        .find_reference(&fetch_ref)
        .with_context(|| format!("Reference `{fetch_ref}` not found"))?
        .peel_to_commit()
        .context("Failed to peel fetched reference to commit")?;

    let new_commit = fetch_commit.id().to_string();
    info!("📋 New commit: {}", new_commit);

    if old_commit == new_commit {
        info!("✅ Repository is already up-to-date");
        return Ok(());
    }

    info!("📊 Commit comparison:");
    info!("  📍 Old: {}", old_commit);
    info!("  📍 New: {}", new_commit);

    // ensure local branch exists
    if repo.find_branch(branch, BranchType::Local).is_err() {
        info!("🌿 Creating local branch: {}", branch);
        repo.branch(branch, &fetch_commit, true)
            .with_context(|| format!("Failed to create local branch `{branch}`"))?;
    }

    info!("🔄 Updating working directory");

    // hard‐reset working tree
    repo.set_head(&format!("refs/heads/{branch}"))
        .with_context(|| format!("Failed to set HEAD to `{branch}`"))?;

    info!("🛠️  Checking out files");
    repo.checkout_head(Some(CheckoutBuilder::default().force()))
        .context("Checkout failed")?;

    info!("🔨 Performing hard reset");
    repo.reset(fetch_commit.as_object(), ResetType::Hard, None)
        .context("Hard reset to fetched commit failed")?;

    let elapsed = start_time.elapsed();
    info!("✅ Repository update completed in {:?}", elapsed);

    // Verify update
    log_commit_info(&fetch_commit, "Updated");
    info!(
        "📅 Time: {} seconds since epoch",
        fetch_commit.time().seconds()
    );

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
