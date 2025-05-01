use actix_web::{
    HttpResponse,
    error::{ErrorConflict, ErrorInternalServerError},
};
use serde::{Deserialize, Serialize};
use std::env;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};
use tokio::task;
use tracing::{debug, error, info};

use crate::utils::folders::thumbnails_dir;
use glob::glob;
use meilisearch_sdk::client::Client;
use shared::types::map_document::MapDocument as MapDoc;
use shared::types::{dd2vtt::DD2VTTFile, map_reference::MapReference};
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
    OpenOptions::new().write(true).create_new(true).open(path)
}

/// Read current lock state
fn read_lock(path: &Path) -> Option<BuildLock> {
    let mut buf = String::new();
    File::open(path).ok()?.read_to_string(&mut buf).ok()?;
    serde_json::from_str(&buf).ok()
}

/// Atomically overwrite lock file
fn write_lock(path: &Path, data: &BuildLock) -> std::io::Result<()> {
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
fn find_dd2vtt_paths() -> Result<Vec<PathBuf>, actix_web::Error> {
    let base = maps_dir().map_err(ErrorInternalServerError)?;
    let pattern = format!("{}/**/*.dd2vtt", base.to_string_lossy());
    debug!("Globbing {}", pattern);

    let mut out = Vec::new();
    for entry in glob(&pattern).map_err(ErrorInternalServerError)? {
        let p = entry.map_err(ErrorInternalServerError)?;
        if p.is_file() {
            out.push(p);
        }
    }
    Ok(out)
}

// process one file
fn process_one(
    path: PathBuf,
    base: PathBuf,
    thumb_dir: PathBuf,
) -> Result<MapReference, anyhow::Error> {
    let dd2vtt = DD2VTTFile::from_path(path);
    let map_ref = MapReference::from(dd2vtt.clone());

    // build thumbnail path
    let orig = dd2vtt.path.clone().unwrap_or_default();
    let rel = orig.strip_prefix(&base)?;
    let mut thumb = thumb_dir.join(rel);
    thumb.set_extension("png");
    if let Some(parent) = thumb.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if !thumb.exists() {
        dd2vtt.export_thumbnail_file(&thumb);
    }

    Ok(map_ref)
}

// collect all references in parallel batches
async fn collect_references() -> Result<Vec<MapReference>, actix_web::Error> {
    let base = maps_dir().map_err(ErrorInternalServerError)?;
    let thumb_dir = thumbnails_dir().map_err(ErrorInternalServerError)?;
    let paths = find_dd2vtt_paths()?;
    info!("Found {} maps", paths.len());

    let mut refs = Vec::new();
    for chunk in paths.chunks(TASK_BATCH_SIZE) {
        let handles: Vec<_> = chunk
            .iter()
            .map(|p| {
                let bd = base.clone();
                let td = thumb_dir.clone();
                let p = p.clone();
                task::spawn_blocking(move || process_one(p, bd, td))
            })
            .collect();

        for h in handles {
            match h.await {
                Ok(Ok(r)) => refs.push(r),
                Ok(Err(e)) => error!("processing error: {:?}", e),
                Err(join_err) => error!("task join error: {:?}", join_err),
            }
        }
    }
    Ok(refs)
}

// git clone & update logic
fn update_repo(path: &Path) -> Result<(), anyhow::Error> {
    let branch = env::var("REPO_REF").unwrap_or_else(|_| "main".into());
    let maps_p = path.join("maps");

    if !maps_p.exists() {
        Command::new("git")
            .args([
                "clone",
                "--branch",
                &branch,
                "https://github.com/mbround18/vtt-maps.git",
                &path.to_string_lossy(),
            ])
            .status()?;
    } else {
        for args in &[
            vec!["fetch", "origin", &branch],
            vec!["checkout", "--force", &branch],
            vec!["pull", "origin", &branch],
        ] {
            Command::new("git").current_dir(path).args(args).status()?;
        }
    }
    Ok(())
}

// main handler
pub async fn maps_rebuild() -> Result<HttpResponse, actix_web::Error> {
    info!("Map rebuild requested");

    let root = root_dir().map_err(ErrorInternalServerError)?;
    let lockfile = lock_path();

    // if lock exists, return its JSON state
    if let Some(state) = read_lock(&lockfile) {
        match state {
            BuildLock::Processing {
                processed, total, ..
            } => {
                return Ok(HttpResponse::Ok().json(serde_json::json!({
                    "status":"processing","processed":processed,"total":total
                })));
            }
            BuildLock::Complete { maps, .. } => {
                return Ok(HttpResponse::Ok().json(serde_json::json!({
                    "status":"up-to-date","processed":maps,"total":maps
                })));
            }
        }
    }

    // prepare workset and lock
    let base = maps_dir().map_err(ErrorInternalServerError)?;
    // let thumb_dir = thumbnails_dir().map_err(ErrorInternalServerError)?;
    let paths = find_dd2vtt_paths()?;
    let total = paths.len();
    let sha = String::from_utf8_lossy(
        &Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&base)
            .output()
            .map_err(ErrorInternalServerError)?
            .stdout,
    )
    .trim()
    .to_string();

    let mut lock =
        try_acquire_lock(&lockfile).map_err(|_| ErrorConflict("Rebuild already running"))?;
    write!(
        lock,
        "{}",
        serde_json::to_string(&BuildLock::Processing {
            processed: 0,
            total,
            sha: sha.clone()
        })
        .unwrap()
    )
    .unwrap();

    // spawn background update
    actix_web::rt::spawn(async move {
        if env::var("REPO_PATH").is_ok() {
            if let Err(e) = update_repo(&root) {
                error!("repo error: {:?}", e);
                return;
            }
        }

        let url = env::var("MEILI_URL").unwrap_or_else(|_| "http://127.0.0.1:7700".into());
        let key = env::var("MEILI_KEY").ok();
        let client = match Client::new(&url, key.as_deref()) {
            Ok(c) => c,
            Err(e) => {
                error!("Meili err: {:?}", e);
                return;
            }
        };
        let index = client.index("maps");

        let refs = match collect_references().await {
            Ok(r) => r,
            Err(e) => {
                error!("collect err: {:?}", e);
                return;
            }
        };
        let docs: Vec<_> = refs
            .into_iter()
            .map(|r| {
                let base_as_str = base.to_string_lossy().to_string();
                let path_relative_to_base = r.path.strip_prefix(&base_as_str).unwrap_or(&r.path);
                let asset_path = path_relative_to_base.replace(".dd2vtt", ".png");

                MapDoc {
                    id: r.hash.clone(),
                    name: r.name,
                    path: format!("/maps{}", path_relative_to_base),
                    thumbnail: format!("/assets/thumbnails{}", asset_path),
                    resolution: r.resolution,
                }
            })
            .collect();

        if let Err(e) = index.delete_all_documents().await {
            error!("delete err: {:?}", e);
            return;
        }
        let mut done = 0;
        for chunk in docs.chunks(TASK_BATCH_SIZE) {
            if let Err(e) = index.add_documents(chunk, Some("id")).await {
                error!("add docs err: {:?}", e);
                return;
            }
            done += chunk.len();
            let _ = write_lock(
                &lockfile,
                &BuildLock::Processing {
                    processed: done,
                    total,
                    sha: sha.clone(),
                },
            );
        }

        let _ = write_lock(&lockfile, &BuildLock::Complete { maps: total, sha });
    });

    Ok(HttpResponse::Accepted().json(serde_json::json!({
        "status":"processing","processed":0,"total":total
    })))
}
