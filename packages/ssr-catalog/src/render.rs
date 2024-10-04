use crate::pages::app::{App, AppProps};
use glob::glob;
use shared::types::map_reference::MapReference;
use shared::utils::root_dir::root_dir;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CatalogRenderError {
    #[error("Failed to find root directory: {0}")]
    RootDirError(#[from] io::Error),

    #[error("Failed to parse glob pattern: {0}")]
    GlobPatternError(#[from] glob::PatternError),

    #[error("Failed to read glob result: {0}")]
    GlobResultError(#[from] glob::GlobError),
}

pub async fn render_catalog() -> Result<String, CatalogRenderError> {
    let root = root_dir()?;
    let glob_path = format!("{}/maps/**/*.info.json", root.to_str().unwrap());
    println!("{}", &glob_path);

    let references = glob(&glob_path)?
        .filter_map(|result| match result {
            Ok(path) => Some(MapReference::from(&path)),
            Err(e) => {
                eprintln!("Failed to read glob result: {}", e);
                None
            }
        })
        .collect::<Vec<MapReference>>();

    let renderer = yew::ServerRenderer::<App>::with_props(|| AppProps { references });
    Ok(renderer.render().await)
}
