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
    let mut list_of_maps = props.references.clone();
    list_of_maps.sort_by(|a, b| a.name.cmp(&b.name));

    let html_references = list_of_maps.iter()
        .map(|item| {
            let name = titlecase(&item.name.replace(['_', '-'], " "));
            let download_url = format!("https://raw.githubusercontent.com/dnd-apps/vtt-maps/main/{}", &item.path);
            let src = format!(
                "data:image/png;base64,{}",
                image_to_base64(&Path::new(&root_dir().unwrap()).join(item.path.replace(".dd2vtt", ".preview.png")))
            );

            html! {
                <>
                    <div class={"card map-asset-view"}>
                        <h1>{name}</h1>
                        <img {src}  class="responsive" />
                        <div>
                            <p><strong>{"Pixels Per Tile: "}</strong>{&item.resolution.pixels_per_grid}</p>
                            <p><strong>{"Tile Length: "}</strong>{&item.resolution.map_size.x}</p>
                            <p><strong>{"Tile Width: "}</strong>{&item.resolution.map_size.y}</p>
                            <button class="download" data-href={download_url}>{"⬇️ Download DD2VTT File"}</button>
                            <br style="padding: 5px;"/>
                            <a href="https://github.com/sponsors/mbround18" target={"_blank"}>{"❤️ Support the Artist"}</a>
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
