use shared::utils::img_to_base64::image_to_base64;
use shared::utils::root_dir::root_dir;
use std::path::Path;
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

#[function_component]
pub fn App(props: &AppProps) -> Html {
    let mut references = props.references.clone();
    references.sort_by(|a, b| a.name.cmp(&b.name));

    let html_references = references.iter()
        .map(|reference| {
            let name = titlecase(&reference.name.replace("_", " ").replace("-", " "));
            let download_url = format!("https://raw.githubusercontent.com/dnd-apps/vtt-maps/main/{}", &reference.path);

            let src = format!(
                "data:image/png;base64,{}",
                image_to_base64(&Path::new(&root_dir().unwrap()).join(&reference.path.replace(".dd2vtt", ".preview.png")))
            );

            html! {
                <>
                    <div class={"card map-asset-view"}>
                        <h1>{name}</h1>
                        <img {src}  class="responsive" />
                        <div>
                            <p><strong>{"Pixels Per Tile: "}</strong>{&reference.resolution.pixels_per_grid}</p>
                            <p><strong>{"Tile Length: "}</strong>{&reference.resolution.map_size.x}</p>
                            <p><strong>{"Tile Width: "}</strong>{&reference.resolution.map_size.y}</p>
                            <button class="download" data-href={download_url}>{"Download DD2VTT File"}</button>
                        </div>

                    </div>
                </>
            }
        })
        .collect::<Html>();

    html! {
        <>
            <div id={"map-asset-vcc"}>
                {html_references}
            </div>
        </>
    }
}
