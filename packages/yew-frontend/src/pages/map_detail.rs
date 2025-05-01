use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, HtmlImageElement, MouseEvent,
    WheelEvent,
};
use yew::prelude::*;

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

#[derive(Clone, PartialEq)]
struct MapViewState {
    scale: f64,
    offset_x: f64,
    offset_y: f64,
    is_dragging: bool,
    last_mouse_x: f64,
    last_mouse_y: f64,
}

#[function_component(MapDetail)]
pub fn map_detail(props: &MapDetailProps) -> Html {
    let name = use_state(String::new);
    let data = use_state(|| None as Option<MapDocument>);
    let loading = use_state(|| LoadingState::Loading);
    let map_view = use_state(|| MapViewState {
        scale: 1.0,
        offset_x: 0.0,
        offset_y: 0.0,
        is_dragging: false,
        last_mouse_x: 0.0,
        last_mouse_y: 0.0,
    });

    let canvas_ref = use_node_ref();
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
                                if let Some(img) = thumb_ref.cast::<HtmlImageElement>() {
                                    let loading_clone = loading.clone();
                                    let onload = Closure::wrap(Box::new(move || {
                                        loading_clone.set(LoadingState::FullMapLoaded);
                                    })
                                        as Box<dyn FnMut()>);

                                    img.set_onload(Some(onload.as_ref().unchecked_ref()));
                                    img.set_src(&format!("/maps/tiled/{}", map_id));
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

    // 3) When full loaded, set up canvas + animation loop
    {
        let canvas_ref = canvas_ref.clone();
        let thumb_ref = thumb_ref.clone();
        let map_view = map_view.clone();
        let loading = loading.clone();

        use_effect(move || {
            if let LoadingState::FullMapLoaded = &*loading {
                // prepare a Rc<RefCell<Closure>> for recursion
                let f_handle: Rc<RefCell<Option<Closure<dyn FnMut()>>>> =
                    Rc::new(RefCell::new(None));
                let f_handle_clone = f_handle.clone();

                // build the closure
                let render_loop = Closure::wrap(Box::new(move || {
                    if let (Some(canvas), Some(img)) = (
                        canvas_ref.cast::<HtmlCanvasElement>(),
                        thumb_ref.cast::<HtmlImageElement>(),
                    ) {
                        let ctx = canvas
                            .get_context("2d")
                            .unwrap()
                            .unwrap()
                            .dyn_into::<CanvasRenderingContext2d>()
                            .unwrap();

                        let state = &*map_view;
                        canvas.set_width(canvas.client_width() as u32);
                        canvas.set_height(canvas.client_height() as u32);

                        ctx.save();
                        ctx.translate(state.offset_x, state.offset_y).unwrap();
                        ctx.scale(state.scale, state.scale).unwrap();
                        ctx.draw_image_with_html_image_element(&img, 0.0, 0.0)
                            .unwrap();
                        ctx.restore();
                    }

                    // schedule next frame
                    if let Some(g) = f_handle_clone.borrow().as_ref() {
                        web_sys::window()
                            .unwrap()
                            .request_animation_frame(g.as_ref().unchecked_ref())
                            .unwrap();
                    }
                }) as Box<dyn FnMut()>);

                // put it into our Rc so it lives on
                *f_handle.borrow_mut() = Some(render_loop);

                // kick off
                if let Some(r) = f_handle.borrow().as_ref() {
                    web_sys::window()
                        .unwrap()
                        .request_animation_frame(r.as_ref().unchecked_ref())
                        .unwrap();
                }
            }
            || ()
        });
    }

    // 4) Pan & zoom handlers
    let onwheel = {
        let map_view = map_view.clone();
        Callback::from(move |e: WheelEvent| {
            e.prevent_default();
            let factor = if e.delta_y() > 0.0 { 0.9 } else { 1.1 };
            let mut state = (*map_view).clone();
            let old_scale = state.scale;
            state.scale = (state.scale * factor).clamp(0.1, 10.0);
            // zoom toward cursor:
            let rect = e
                .target_unchecked_into::<HtmlElement>()
                .get_bounding_client_rect();
            let mx = e.client_x() as f64 - rect.left();
            let my = e.client_y() as f64 - rect.top();
            let world_x = (mx - state.offset_x) / old_scale;
            let world_y = (my - state.offset_y) / old_scale;
            state.offset_x += world_x * old_scale - world_x * state.scale;
            state.offset_y += world_y * old_scale - world_y * state.scale;
            map_view.set(state);
        })
    };

    let onmousedown = {
        let map_view = map_view.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let rect = e
                .target_unchecked_into::<HtmlElement>()
                .get_bounding_client_rect();
            let x = e.client_x() as f64 - rect.left();
            let y = e.client_y() as f64 - rect.top();
            let mut state = (*map_view).clone();
            state.is_dragging = true;
            state.last_mouse_x = x;
            state.last_mouse_y = y;
            map_view.set(state);
        })
    };

    let onmousemove = {
        let map_view = map_view.clone();
        Callback::from(move |e: MouseEvent| {
            if map_view.is_dragging {
                e.prevent_default();
                let rect = e
                    .target_unchecked_into::<HtmlElement>()
                    .get_bounding_client_rect();
                let x = e.client_x() as f64 - rect.left();
                let y = e.client_y() as f64 - rect.top();
                let mut state = (*map_view).clone();
                state.offset_x += x - state.last_mouse_x;
                state.offset_y += y - state.last_mouse_y;
                state.last_mouse_x = x;
                state.last_mouse_y = y;
                map_view.set(state);
            }
        })
    };

    let end_drag = {
        let map_view = map_view.clone();
        Callback::from(move |_| {
            let mut state = (*map_view).clone();
            state.is_dragging = false;
            map_view.set(state);
        })
    };

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
                                    <p>{"Loading full map..."}</p>
                                    <img
                                        src={map.thumbnail.clone()}
                                        class="responsive"
                                        alt={format!("{} thumbnail", map.name)}
                                    />
                                    // preload full tiled image
                                    <img
                                        ref={thumb_ref.clone()}
                                        style="display:none;"
                                    />
                                </div>
                            }
                        } else { html!{ <p>{"Waiting for data..."}</p> } }
                    }
                    LoadingState::FullMapLoaded => {
                        html! {
                            <div id="map-viewer">
                                <h1>{ &*name }</h1>
                                <div class="map-canvas-container">
                                    <canvas
                                        ref={canvas_ref.clone()}
                                        {onwheel}
                                        {onmousedown}
                                        onmousemove={onmousemove}
                                        onmouseup={end_drag.clone()}
                                        onmouseleave={end_drag}
                                        style="width:100%;height:80vh;cursor:move;"
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
