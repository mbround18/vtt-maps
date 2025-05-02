use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlImageElement;
use yew::prelude::*;

use crate::utils::capitalize;
use shared::types::map_document::MapDocument;
use crate::api::api::ApiEndpoint;

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
    let name = use_state(|| String::new());
    let data = use_state(|| None as Option<MapDocument>);
    let loading = use_state(|| LoadingState::Loading);
    let thumb_ref = use_node_ref();

    // pull your endpoints up-front
    let metadata_ep = ApiEndpoint::GetMap { id: props.id.clone() };
    let tiled_ep    = ApiEndpoint::GetTiledMap { id: props.id.clone() };

    // 1) Fetch metadata when `props.id` changes
    {
        let metadata_ep = metadata_ep.clone();
        let name = name.clone();
        let data = data.clone();
        let loading = loading.clone();

        use_effect_with(
            props.id.clone(),
            move |_map_id| {
                let metadata_ep = metadata_ep.clone();
                let name = name.clone();
                let data = data.clone();
                let loading = loading.clone();

                spawn_local(async move {
                    match metadata_ep.request().send().await {
                        Ok(resp) if resp.status() == 200 => {
                            match resp.json::<MapDocument>().await {
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
                                Err(_) => loading.set(LoadingState::Error("Failed to parse metadata".into())),
                            }
                        }
                        _ => loading.set(LoadingState::Error("Failed to fetch metadata".into())),
                    }
                });

                || ()
            },
        );
    }

    // 2) Preload full tiled map when thumbnail has loaded
    {
        let tiled_ep = tiled_ep.clone();
        let loading = loading.clone();
        let thumb_ref = thumb_ref.clone();

        use_effect_with(
            loading.clone(),
            move |state| {
                if **state == LoadingState::ThumbnailLoaded {
                    let tiled_ep = tiled_ep.clone();
                    let loading = loading.clone();
                    let thumb_ref = thumb_ref.clone();

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
            },
        );
    }

    // 3) Render
    html! {
        <div id="map-container">
            {
                match &*loading {
                    LoadingState::Loading => html! {
                        <p>{"Loading map..."}</p>
                    },
                    LoadingState::ThumbnailLoaded | LoadingState::FullMapLoading => {
                        if let Some(map) = &*data {
                            html! {
                                <div id="map-asset-view">
                                    <h1>{ &*name }</h1>
                                    <p class="loading-text">
                                        {"Loading full map"}<span class="loading-dots"></span>
                                    </p>
                                    <img
                                        src={ map.thumbnail.clone() }
                                        class="responsive"
                                        alt={ format!("{} thumbnail", map.name) }
                                    />
                                    <img
                                        ref={ thumb_ref.clone() }
                                        style="display:none;"
                                        alt="preload full map"
                                    />
                                </div>
                            }
                        } else {
                            html! { <p>{"Waiting for metadata..."}</p> }
                        }
                    }
                    LoadingState::FullMapLoaded => html! {
                        <div id="map-viewer">
                            <h1>{ &*name }</h1>
                            <div class="map-scroll-container">
                                <img
                                    src={ tiled_ep.url() }
                                    class="responsive"
                                    alt={ format!("{} full map", &*name) }
                                />
                            </div>
                        </div>
                    },
                    LoadingState::Error(msg) => html! {
                        <div class="error">{ msg }</div>
                    }
                }
            }
        </div>
    }
}
