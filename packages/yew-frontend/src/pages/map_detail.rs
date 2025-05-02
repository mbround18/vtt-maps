use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::HtmlImageElement;
use yew::prelude::*;

use crate::utils::api::api_base_url;
use crate::utils::{api::get, capitalize};
use shared::types::map_document::MapDocument;

#[derive(Properties, PartialEq)]
pub struct MapDetailProps {
    pub id: String,
}

#[derive(Clone, PartialEq)]
enum LoadingState {
    Loading,
    ThumbnailLoaded,
    FullMapLoading,
    FullMapLoaded,
    Error(String),
}

#[function_component(MapDetail)]
pub fn map_detail(props: &MapDetailProps) -> Html {
    let name = use_state(String::new);
    let data = use_state(|| None as Option<MapDocument>);
    let loading = use_state(|| LoadingState::Loading);
    let full_url = format!("{}/api/maps/tiled/{}", api_base_url(), props.id);

    let thumb_ref = use_node_ref();

    // 1) Fetch map metadata when `props.id` changes
    {
        let id = props.id.clone();
        let data = data.clone();
        let name = name.clone();
        let loading = loading.clone();

        use_effect_with(id.clone(), move |map_id| {
            let data = data.clone();
            let name = name.clone();
            let loading = loading.clone();
            let map_id = map_id.clone(); // Clone the map_id to move into async block

            wasm_bindgen_futures::spawn_local(async move {
                match get(&format!("/maps/{}", map_id)).send().await {
                    Ok(resp) if resp.status() == 200 => match resp.json::<MapDocument>().await {
                        Ok(map) => {
                            let pretty = map
                                .name
                                .split('-')
                                .map(capitalize)
                                .collect::<Vec<_>>()
                                .join(" ");
                            name.set(pretty);
                            data.set(Some(map));
                            loading.set(LoadingState::ThumbnailLoaded);
                        }
                        Err(_) => loading.set(LoadingState::Error("Parse error".into())),
                    },
                    _ => loading.set(LoadingState::Error("Fetch error".into())),
                }
            });

            // cleanup
            || ()
        });
    }

    // 2) When thumbnail's loaded, fetch the full map
    {
        let data = data.clone();
        let loading = loading.clone();
        let thumb_ref = thumb_ref.clone();

        use_effect_with(loading.clone(), move |current_state| {
            if let LoadingState::ThumbnailLoaded = **current_state {
                if let Some(map) = (*data).clone() {
                    let loading = loading.clone();
                    let map_id = map.id;
                    let thumb_ref = thumb_ref.clone();

                    wasm_bindgen_futures::spawn_local(async move {
                        loading.set(LoadingState::FullMapLoading);
                        match get(&format!("/maps/tiled/{}", map_id)).send().await {
                            Ok(resp) if resp.status() == 200 => {
                                let full_url =
                                    format!("{}/api/maps/tiled/{}", api_base_url(), map_id);

                                if let Some(img) = thumb_ref.cast::<HtmlImageElement>() {
                                    let loading_clone = loading.clone();
                                    let onload = Closure::wrap(Box::new(move || {
                                        loading_clone.set(LoadingState::FullMapLoaded);
                                    })
                                        as Box<dyn FnMut()>);

                                    img.set_onload(Some(onload.as_ref().unchecked_ref()));
                                    img.set_src(&full_url);
                                    onload.forget();
                                }
                            }
                            _ => loading.set(LoadingState::Error("Full map failed".into())),
                        }
                    });
                }
            }

            || ()
        });
    }

    // final render
    html! {
        <div id="map-container">
            {
                match &*loading {
                    LoadingState::Loading => html!{ <p>{"Loading map..."}</p> },
                    LoadingState::ThumbnailLoaded | LoadingState::FullMapLoading => {
                        if let Some(map) = &*data {
                            html! {
                                <div id="map-asset-vcc">
                                    <h1>{ &*name }</h1>
                                    <p class="loading-text">{"Loading full map"}<span class="loading-dots"></span></p>
                                    <img
                                        src={map.thumbnail.clone()}
                                        class="responsive"
                                        alt={format!("{} thumbnail", map.name)}
                                    />
                                    // preload full tiled image with thumb_ref - hidden until loaded
                                    <img
                                        ref={thumb_ref.clone()}
                                        style="display:none;"
                                        alt={format!("{} full map", map.name)}
                                    />
                                </div>
                            }
                        } else { html!{ <p>{"Waiting for data..."}</p> } }
                    }
                    LoadingState::FullMapLoaded => {
                        html! {
                            <div id="map-viewer">
                                <h1>{ &*name }</h1>
                                <div class="map-scroll-container">
                                    <img
                                        src={full_url}
                                        class="responsive"
                                        alt={format!("{} full map", &*name)}
                                    />
                                </div>
                            </div>
                        }
                    }
                    LoadingState::Error(msg) => html! {
                        <div class="error">{ msg }</div>
                    },
                }
            }
        </div>
    }
}
