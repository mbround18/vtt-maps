// src/components/map_downloader.rs
use crate::api::context::ApiEndpoint;
use gloo_console::log;
use serde::Deserialize;
use yew::prelude::*;

#[derive(Clone, PartialEq, Deserialize)]
pub struct MapDetails {
    pub id: String,
    pub name: String,
    pub resolution: Resolution,
}

#[derive(Clone, PartialEq, Deserialize)]
pub struct Resolution {
    pub map_size: Size,
    pub pixels_per_grid: u32,
}

#[derive(Clone, PartialEq, Deserialize)]
pub struct Size {
    pub x: u32,
    pub y: u32,
}

#[derive(Properties, PartialEq)]
pub struct MapDownloaderProps {
    pub id: String,
}

#[function_component(MapDownloader)]
pub fn map_downloader(props: &MapDownloaderProps) -> Html {
    let details = use_state(|| None::<MapDetails>);
    let id = props.id.clone();
    let request = ApiEndpoint::Map { id: id.clone() };

    {
        let details = details.clone();
        use_effect_with(id.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match request.request().send().await {
                    Ok(resp) if resp.ok() => match resp.json::<MapDetails>().await {
                        Ok(map) => details.set(Some(map)),
                        Err(e) => log!("Failed to parse map details:", e.to_string()),
                    },
                    Ok(resp) => log!("Fetch failed:", resp.status()),
                    Err(e) => log!("Network error:", e.to_string()),
                }
            });
            || ()
        });
    }

    if let Some(map) = &*details {
        // build URLs & dimension text
        let dd2vtt_url = format!("/api/maps/download/{}", map.id);
        let img_url = format!("/api/maps/tiled/{}", map.id);
        let download_name = map
            .name
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("-")
            + ".png";
        let dims = {
            html! {
                <div class="flex flex-col gap-2">
                    <span>{ "Map Dimensions:" }</span>
                    <span class="pl-4">
                        { format!("Squares: {} × {}", map.resolution.map_size.x, map.resolution.map_size.y)  }
                    </span>
                    <span class="pl-4">
                        { format!("Pixels Per Square: {}", map.resolution.pixels_per_grid) }
                    </span>
                </div>
            }
        };

        html! {
            <div id="action-buttons" class="map-downloader rounded">
                <div class="flex flex-row gap-2">
                    <div>
                        <a href={dd2vtt_url} download={"true"} class="btn btn-primary">
                            { "Download DD2VTT File" }
                        </a>
                    </div>
                    <div>
                    <a href={img_url} download={download_name} class="btn btn-primary">
                        { "Download Image" }
                    </a>
                   </div>
               </div>
                <div class="space-y-1 pt-2">
                  <p class="text-sm m-0">
                    <span class="font-bold">{"DD2VTT"}</span> {" Files are used to import into your VTT of choice!"}
                  </p>
                  <p class="text-sm m-0">
                    {"For FoundryVTT, you will need this module:"}
                    <a
                      href="https://foundryvtt.com/packages/dd-import/"
                      target="_blank"
                      rel="noopener noreferrer"
                      class="text-blue-600 hover:underline pl-1"
                    >
                      {"Universal Battlemap Importer"}
                    </a>
                  </p>
                </div>
                <div class="map-dimensions pt-2">{ dims }</div>
            </div>
        }
    } else {
        html! { <p>{ "Loading map metadata..." }</p> }
    }
}
