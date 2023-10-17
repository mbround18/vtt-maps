mod pages;

use crate::pages::app::{App, AppProps};
use glob::glob;
use shared::types::map_reference::MapReference;
use shared::utils::root_dir::root_dir;
use std::path::Path;

#[tokio::main]
async fn main() {
    // let bytes = include_bytes!("../index.html") as &[u8];
    // let index = String::from_utf8(Vec::from(bytes)).unwrap();
    let glob_path = format!(
        "{}/**/*.info.json",
        &root_dir()
            .expect("Failed to find root dir")
            .to_str()
            .unwrap()
    );
    println!("{}", &glob_path);
    let references = glob(&glob_path)
        .unwrap()
        .map(|path| MapReference::from(&path.unwrap()))
        .collect::<Vec<MapReference>>();

    let renderer = yew::ServerRenderer::<App>::with_props(|| AppProps { references });
    let rendered = renderer.render().await;

    // Prints: <div>Hello, World!</div>
    // println!("{}", &rendered);
    let dist_path = Path::new(&root_dir().expect("Failed to find root dir"))
        .join("packages/gh-pagify/dist/assets");
    if !dist_path.exists() {
        std::fs::create_dir_all(&dist_path).expect("Failed to create dist dir");
    }

    let output = dist_path.join("catalog.html");
    std::fs::write(output, rendered).expect("Unable to write file");
}
