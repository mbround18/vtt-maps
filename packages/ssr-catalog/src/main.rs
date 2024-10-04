mod pages;
mod render;

use shared::utils::root_dir::root_dir;
use std::fs;

#[tokio::main]
async fn main() {
    let root_directory = match root_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Error finding root directory: {}", e);
            return;
        }
    };

    let dist_path = root_directory.join("dist/assets");
    println!("Path: {dist_path:?}");

    if let Err(e) = fs::create_dir_all(&dist_path) {
        eprintln!("Failed to create dist directory: {}", e);
        return;
    }

    let rendered = match render::render_catalog().await {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error rendering catalog: {}", e);
            return;
        }
    };

    let output = dist_path.join("catalog.html");
    if let Err(e) = fs::write(&output, rendered.as_bytes()) {
        eprintln!("Unable to write file {}: {}", output.display(), e);
    } else {
        println!("Catalog successfully written to: {}", output.display());
    }
}
