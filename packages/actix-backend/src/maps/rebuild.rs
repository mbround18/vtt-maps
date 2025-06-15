use actix_web::{HttpResponse, error::ErrorInternalServerError};
use serde::{Deserialize, Serialize};
use std::env;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};
use tokio::task;
use tracing::{debug, error, info, instrument, warn};

use crate::utils::folders::thumbnails_dir;
use crate::utils::repo::{get_sha, update_repo};
use glob::glob;
use meilisearch_sdk::client::Client;
use shared::types::map_document::MapDocument as MapDoc;
use shared::types::{dd2vtt::DD2VTTFile, map_reference::MapReference};
use shared::utils::casing::titlecase;
use shared::utils::root_dir::{maps_dir, root_dir};

const TASK_BATCH_SIZE: usize = 50;

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum BuildLock {
    Processing {
        processed: usize,
        total: usize,
        sha: String,
    },
    Complete {
        maps: usize,
        sha: String,
    },
}

fn lock_path() -> PathBuf {
    thumbnails_dir().unwrap().join(".map_rebuild_lock.json")
}

/// Atomically create lock (fails if exists)
fn try_acquire_lock(path: &Path) -> std::io::Result<File> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    OpenOptions::new().write(true).create_new(true).open(path)
}

/// Read current lock state
fn read_lock(path: &Path) -> Option<BuildLock> {
    let mut file = File::open(path).ok()?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).ok()?;
    drop(file); // Explicit file handle cleanup
    serde_json::from_str(&buf).ok()
}

/// Atomically overwrite lock file
fn write_lock(path: &Path, data: &BuildLock) -> std::io::Result<()> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let tmp = path.with_extension("lock.tmp");
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;
    write!(f, "{}", serde_json::to_string(data).unwrap())?;
    std::fs::rename(tmp, path)?;
    Ok(())
}

// find all .dd2vtt files
#[instrument(level = "debug")]
fn find_dd2vtt_paths() -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let start = Instant::now();
    let base = maps_dir()?;

    if !base.exists() {
        error!("Maps directory not found: {}", base.display());
        return Err(format!("Maps directory not found: {}", base.display()).into());
    }

    let pattern = format!("{}/**/*.dd2vtt", base.to_string_lossy());
    info!("🔍 Scanning for DD2VTT files with pattern: {}", pattern);

    let mut out = Vec::new();
    for entry in glob(&pattern)? {
        let p = entry?;
        if p.is_file() {
            debug!("📄 Found DD2VTT file: {}", p.display());
            out.push(p);
        }
    }

    let elapsed = start.elapsed();
    info!("✅ Discovered {} DD2VTT files in {:?}", out.len(), elapsed);
    Ok(out)
}

// process one file
#[instrument(level = "debug", fields(file = %path.display()))]
fn process_one(
    path: PathBuf,
    base: PathBuf,
    thumb_dir: PathBuf,
) -> Result<MapReference, anyhow::Error> {
    debug!("🔄 Processing map file: {}", path.display());
    let dd2vtt = DD2VTTFile::from_path(path.clone());

    let default_path = PathBuf::new();
    let orig = dd2vtt.path.as_ref().unwrap_or(&default_path);
    let rel = orig.strip_prefix(&base)?;
    let mut thumb = thumb_dir.join(rel);
    thumb.set_extension("png");

    if let Some(parent) = thumb.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if !thumb.exists() {
        debug!("🖼️  Generating thumbnail: {}", thumb.display());
        dd2vtt.clone().export_thumbnail_file(&thumb);
        debug!("✅ Thumbnail generated: {}", thumb.display());
    } else {
        debug!("♻️  Thumbnail already exists: {}", thumb.display());
    }

    let map_ref = MapReference::from(dd2vtt);
    debug!("✅ Processed map: {} ({})", map_ref.name, map_ref.hash);
    Ok(map_ref)
}

/// Convert MapReference to MapDocument efficiently
fn map_ref_to_doc(map_ref: MapReference, base_path: &str) -> MapDoc {
    let path_relative_to_base = map_ref
        .path
        .strip_prefix(base_path)
        .unwrap_or(&map_ref.path);
    let asset_path = path_relative_to_base.replace(".dd2vtt", ".png");
    let content_path = path_relative_to_base.replace(".dd2vtt", ".md");

    MapDoc {
        id: map_ref.hash,
        name: titlecase(&map_ref.name),
        path: format!("/maps{}", path_relative_to_base),
        thumbnail: format!("/assets/thumbnails{}", asset_path),
        content: {
            let full_path = format!("{}/{}", base_path, content_path);
            if Path::new(&full_path).exists() {
                Some(format!("/maps{}", content_path))
            } else {
                None
            }
        },
        resolution: map_ref.resolution,
    }
}

/// Core rebuild function that can be called from anywhere
#[instrument(level = "info")]
pub async fn rebuild_maps_core() -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let start = Instant::now();
    info!("🚀 Starting map rebuild process");

    let root = root_dir()?;
    std::fs::create_dir_all(&root)?;
    info!("📁 Root directory ready: {}", root.display());

    let lockfile = lock_path();

    // Only prevent concurrent rebuilds, but always check index/documents
    if let Some(BuildLock::Processing {
        processed, total, ..
    }) = read_lock(&lockfile)
    {
        warn!(
            "⚠️  Rebuild already in progress: {}/{} maps processed",
            processed, total
        );
        return Err("Rebuild already in progress".into());
    }

    let base = maps_dir()?;
    std::fs::create_dir_all(&base)?;

    let paths = find_dd2vtt_paths()?;
    let total = paths.len();
    info!("📊 Planning to process {} total maps", total);

    let sha = get_sha()?;
    info!("📋 Current SHA: {}", sha);

    // Acquire lock
    let mut lock = match try_acquire_lock(&lockfile) {
        Ok(lock) => lock,
        Err(_) => return Err("Rebuild already running".into()),
    };

    write!(
        lock,
        "{}",
        serde_json::to_string(&BuildLock::Processing {
            processed: 0,
            total,
            sha: sha.clone()
        })?
    )?;
    info!("🔒 Lock acquired, starting rebuild");

    // Update repository if configured
    if env::var("REPO_DIR").is_ok() {
        info!("📥 Updating repository");
        if let Err(e) = update_repo() {
            error!("❌ Repository update failed: {:?}", e);
            return Err(format!("Repository update failed: {:?}", e).into());
        }
        info!("✅ Repository updated");
    }

    // Initialize Meilisearch client
    let url = env::var("MEILI_URL").unwrap_or_else(|_| "http://127.0.0.1:7700".into());
    let key = env::var("MEILI_KEY").ok();
    info!("🔗 Connecting to Meilisearch at: {}", url);

    let client = Client::new(&url, key.as_deref())?;
    let index = client.index("maps");

    // Always check if 'maps' index exists, create if missing
    info!("🔍 Checking if 'maps' index exists");
    match index.get_stats().await {
        Ok(_) => info!("✅ Index 'maps' already exists"),
        Err(_) => {
            info!("🏗️  Creating 'maps' index");
            match client.create_index("maps", Some("id")).await {
                Ok(task) => {
                    info!("📋 Index creation task submitted: {}", task.task_uid);
                    let _ = task.wait_for_completion(&client, None, None).await?;
                    info!("✅ Index 'maps' created successfully");
                }
                Err(e) => {
                    error!("❌ Failed to create index 'maps': {:?}", e);
                    return Err(format!("Failed to create index: {:?}", e).into());
                }
            }
        }
    }

    // Always rebuild documents in streaming batches to minimize memory usage
    info!("�️  Clearing existing documents from search index");
    index.delete_all_documents().await?;
    info!("✅ Search index cleared");

    info!("🔄 Processing and indexing maps in streaming batches");
    let base_as_str = base.to_string_lossy().to_string();
    let thumb_dir = thumbnails_dir()?;
    let paths = find_dd2vtt_paths()?;
    let total = paths.len();
    let total_batches = if total == 0 {
        0
    } else {
        total.div_ceil(TASK_BATCH_SIZE)
    };

    let mut processed = 0;
    for (batch_idx, chunk) in paths.chunks(TASK_BATCH_SIZE).enumerate() {
        info!(
            "📊 Processing and indexing batch {}/{} ({} maps)",
            batch_idx + 1,
            total_batches,
            chunk.len()
        );

        let handles: Vec<_> = chunk
            .iter()
            .map(|p| {
                let bd = base.clone();
                let td = thumb_dir.clone();
                let p = p.clone();
                task::spawn_blocking(move || process_one(p, bd, td))
            })
            .collect();

        let mut batch_docs = Vec::with_capacity(chunk.len());
        for h in handles {
            match h.await {
                Ok(Ok(map_ref)) => {
                    let doc = map_ref_to_doc(map_ref, &base_as_str);
                    batch_docs.push(doc);
                }
                Ok(Err(e)) => error!("❌ Processing error: {:?}", e),
                Err(join_err) => error!("⚠️  Task join error: {:?}", join_err),
            }
        }

        if !batch_docs.is_empty() {
            info!("� Indexing {} documents", batch_docs.len());
            index.add_documents(&batch_docs, Some("id")).await?;
            processed += batch_docs.len();

            write_lock(
                &lockfile,
                &BuildLock::Processing {
                    processed,
                    total,
                    sha: sha.clone(),
                },
            )?;
        }

        // Explicitly drop the batch to free memory
        drop(batch_docs);

        info!(
            "✅ Batch {}/{} processed and indexed",
            batch_idx + 1,
            total_batches
        );
    }
    write_lock(&lockfile, &BuildLock::Complete { maps: total, sha })?;
    let total_elapsed = start.elapsed();
    info!(
        "🎉 Map rebuild completed successfully: {} maps processed in {:?}",
        total, total_elapsed
    );
    Ok(total)
}

// main handler
pub async fn maps_rebuild() -> Result<HttpResponse, actix_web::Error> {
    info!("🌐 Map rebuild requested via HTTP endpoint");

    let lockfile = lock_path();

    // Only prevent concurrent rebuilds, but always check index/documents
    if let Some(BuildLock::Processing {
        processed, total, ..
    }) = read_lock(&lockfile)
    {
        info!(
            "📊 Rebuild in progress: {}/{} maps processed",
            processed, total
        );
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "status":"processing","processed":processed,"total":total
        })));
    }

    // Start rebuild in background
    actix_web::rt::spawn(async move {
        if let Err(e) = rebuild_maps_core().await {
            error!("❌ Background rebuild failed: {:?}", e);
            let _ = std::fs::remove_file(&lockfile);
        }
    });

    let paths = find_dd2vtt_paths().map_err(|e| {
        error!("Failed to scan for maps: {:?}", e);
        ErrorInternalServerError("Failed to scan for maps")
    })?;
    let total = paths.len();

    info!("🚀 Background rebuild started for {} maps", total);
    Ok(HttpResponse::Accepted().json(serde_json::json!({
        "status":"processing","processed":0,"total":total
    })))
}
