use crate::api::api::ApiEndpoint;
use crate::components::map_downloader::MapDownloader;
use shared::types::map_document::MapDocument;
use shared::utils::casing::titlecase;

use gloo_console::info;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlImageElement;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq)]
enum LoadingState {
    Loading,
    ThumbnailLoaded,
    FullMapLoading,
    FullMapLoaded,
    Error(String),
}

#[derive(Properties, PartialEq)]
pub struct MapDetailProps {
    pub id: String,
}

#[function_component(MapDetail)]
pub fn map_detail(props: &MapDetailProps) -> Html {
    // --- state hooks ---
    let name = use_state(String::new);
    let data = use_state(|| None as Option<MapDocument>);
    let loading = use_state(|| LoadingState::Loading);
    let thumb_ref = use_node_ref();
    let content = use_state(String::new);

    // --- API endpoints ---
    let metadata_ep = ApiEndpoint::GetMap {
        id: props.id.clone(),
    };
    let tiled_ep = ApiEndpoint::GetTiledMap {
        id: props.id.clone(),
    };

    // 1) Fetch metadata when `props.id` changes
    {
        let metadata_ep = metadata_ep.clone();
        let name = name.clone();
        let data = data.clone();
        let loading = loading.clone();

        use_effect_with(props.id.clone(), move |_| {
            spawn_local(async move {
                match metadata_ep.request().send().await {
                    Ok(resp) if resp.status() == 200 => {
                        if let Ok(map) = resp.json::<MapDocument>().await {
                            let pretty = map
                                .name
                                .split('-')
                                .map(titlecase)
                                .collect::<Vec<_>>()
                                .join(" ");

                            name.set(pretty);
                            data.set(Some(map));
                            loading.set(LoadingState::ThumbnailLoaded);
                        } else {
                            loading.set(LoadingState::Error("Failed to parse metadata".into()));
                        }
                    }
                    _ => {
                        loading.set(LoadingState::Error("Failed to fetch metadata".into()));
                    }
                }
            });

            || ()
        });
    }

    // 2) Preload full tiled map when thumbnail has loaded
    {
        let tiled_ep = tiled_ep.clone();
        let loading = loading.clone();
        let thumb_ref = thumb_ref.clone();

        use_effect_with(loading.clone(), move |state| {
            if **state == LoadingState::ThumbnailLoaded {
                spawn_local(async move {
                    loading.set(LoadingState::FullMapLoading);

                    if let Some(img) = thumb_ref.cast::<HtmlImageElement>() {
                        let onload = Closure::wrap(Box::new(move || {
                            loading.set(LoadingState::FullMapLoaded);
                        }) as Box<dyn FnMut()>);

                        img.set_onload(Some(onload.as_ref().unchecked_ref()));
                        img.set_src(&tiled_ep.url());
                        onload.forget();
                    } else {
                        loading.set(LoadingState::Error("Preload element missing".into()));
                    }
                });
            }

            || ()
        });
    }

    // 3) Fetch map content markdown
    {
        let content = content.clone();
        use_effect_with(props.id.clone(), move |id: &String| {
            let id = id.clone();

            spawn_local(async move {
                let request = ApiEndpoint::GetMapContent { id };

                match request.request().send().await {
                    Ok(resp) if resp.status() == 200 => {
                        if let Ok(txt) = resp.text().await {
                            content.set(txt);
                        } else {
                            info!("Failed to read body");
                        }
                    }
                    Ok(resp) => {
                        info!("Fetch failed: {}", resp.status());
                    }
                    Err(err) => {
                        info!("Network error: {}", err.to_string());
                    }
                }
            });

            || ()
        });
    }

    // --- render ---
    html! {
        <div id="map-container" class="map-container">
            <div id="map-asset-view" class="map-asset-view">
                {
                    match &*loading {
                        LoadingState::Loading => html! {
                            <div>
                                <h1 id="map-title" class="map-title">{ &*name }</h1>
                                <p id="loading-text" class="loading-text loading-message">
                                    {"Loading full map"}<span class="loading-dots"></span>
                                </p>
                            </div>
                        },

                        LoadingState::ThumbnailLoaded | LoadingState::FullMapLoading => {
                            if let Some(map) = &*data {
                                html! {
                                    <div>
                                        <h1 id="map-title" class="map-title">{ &*name }</h1>
                                        <p id="loading-text" class="loading-text loading-message">
                                            {"Loading full map"}<span class="loading-dots"></span>
                                        </p>
                                        <img
                                            id="map-thumbnail"
                                            src={ map.thumbnail.clone() }
                                            class="map-thumbnail responsive"
                                            alt={ format!("{} thumbnail", map.name) }
                                        />
                                        <img
                                            id="preload-image"
                                            ref={ thumb_ref.clone() }
                                            class="preload-image"
                                            style="display:none;"
                                            alt="preload full map"
                                        />
                                    </div>
                                }
                            } else {
                                html! {
                                    <p id="metadata-wait" class="metadata-wait">
                                        {"Waiting for metadata..."}
                                    </p>
                                }
                            }
                        },

                        LoadingState::FullMapLoaded => {
                            let inner = Html::from_html_unchecked(
                                AttrValue::from((*content).clone())
                            );

                            let loaded_map =html! {
                                <div id="map-scroll-container" class="map-scroll-container">
                                    <img
                                        id="full-map-image"
                                        src={ tiled_ep.url() }
                                        class="responsive pointer-events-none full-map"
                                        alt={ format!("{} full map", &*name) }
                                    />
                                </div>
                            };

                            if content.is_empty() {
                                html! {
                                    <div id="map-viewer" class="map-viewer">
                                        <h1 id="map-title" class="map-title">
                                            { &*name }
                                        </h1>
                                        { loaded_map }
                                    </div>
                                }
                            } else {
                                html! {
                                    <div id="map-viewer" class="map-viewer">
                                        { loaded_map }
                                        <div id="map-content" class="map-content markdown">
                                            { inner }
                                        </div>
                                    </div>
                                }
                            }
                        },

                        LoadingState::Error(msg) => html! {
                            <div id="error-message" class="error-message error">
                                { msg }
                            </div>
                        },
                    }
                }
               <div id="map-download" class="flex flex-col gap-2">
                    <div>
                        <MapDownloader id={ props.id.clone() } />
                    </div>
                    <div>
                     <iframe
                        id={"kofiframe"}
                        src={"https://ko-fi.com/mbround18/?hidefeed=true&widget=true&embed=true&preview=true"}
                        style={"border:none;width:100%;padding:4px;background:#f9f9f9;"}
                        height={"712"}
                        title="mbround18"
                        ></iframe>
                    </div>
                </div>
            </div>
        </div>
    }
}
