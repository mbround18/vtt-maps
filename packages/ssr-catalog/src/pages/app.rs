use shared::utils::img_to_base64::image_to_base64;
use shared::utils::root_dir::root_dir;
use std::path::{Path, PathBuf};
use titlecase::titlecase;
use wasm_bindgen::prelude::wasm_bindgen;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn handleDownload(s: &str);
}

#[derive(Properties, Clone, PartialEq)]
pub struct AppProps {
    pub references: Vec<shared::types::map_reference::MapReference>,
}

#[function_component(App)]
pub fn app(props: &AppProps) -> Html {
    let mut list_of_maps = props.references.clone();
    list_of_maps.sort_by(|a, b| a.name.cmp(&b.name));

    let html_references = list_of_maps.iter()
      .map(|item| {
          let name = titlecase(&item.name.replace(['_', '-'], " "));
          let download_url = {
            let download_path = item.path.strip_prefix(root_dir().unwrap().to_str().unwrap()).unwrap_or("README.md");
            format!("https://raw.githubusercontent.com/dnd-apps/vtt-maps/main/{download_path}").replace("//", "/")
          };

          // Correct the path to ensure it is always looking in the "maps" directory
          let path_without_extension: PathBuf = Path::new(&item.path).with_extension("");
          let preview_file_path = path_without_extension.with_extension("preview.png");
          let preview_path = image_to_base64(&preview_file_path);

          let src = format!(
              "data:image/png;base64,{}",
              preview_path.unwrap_or_else(|_| String::from("")
              ));

          let cloned_download_url = download_url.clone();

          html! {
                <div class="card map-asset-view">
                    <h1>{name}</h1>
                    <img {src} class="responsive" />
                    <div>
                        <p><strong>{"Pixels Per Tile: "}</strong>{&item.resolution.pixels_per_grid}</p>
                        <p><strong>{"Tile Length: "}</strong>{&item.resolution.map_size.x}</p>
                        <p><strong>{"Tile Width: "}</strong>{&item.resolution.map_size.y}</p>
                        <button class="download" data-href={download_url} onclick={Callback::from(move |_| handleDownload(&cloned_download_url))}>{"⬇️ Download DD2VTT File"}</button>
                        <br style="padding: 5px;"/>
                        <a href="https://github.com/sponsors/mbround18" target="_blank">{"❤️ Support the Artist"}</a>
                    </div>
                </div>
            }
      })
      .collect::<Html>();

    html! {
        <div id="map-asset-vcc">
            {html_references}
        </div>
    }
}
