use crate::components::map_asset_card::MapAssetCard;
use shared::types::map_document::MapDocument;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::window;
use yew::prelude::*;
use yew_hooks::prelude::*;

const PAGE_SIZE: usize = 25;

#[function_component(Catalog)]
pub fn catalog() -> Html {
    let maps = use_state(Vec::<MapDocument>::new);
    let offset = use_state(|| 0usize);
    let is_loading = use_state(|| false);
    let has_more = use_state(|| true);

    // Debounced effect to load more maps
    {
        let maps = maps.clone();
        let offset = offset.clone();
        let is_loading = is_loading.clone();
        let has_more = has_more.clone();

        use_debounce_effect(
            move || {
                if *is_loading || !*has_more {
                    return;
                }

                is_loading.set(true);
                let maps = maps.clone();
                let offset = offset.clone();
                let is_loading = is_loading.clone();
                let has_more = has_more.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/maps/all?offset={}&limit={}", *offset, PAGE_SIZE);
                    if let Ok(resp) = crate::utils::api::get(&url).send().await {
                        if let Ok(mut list) = resp.json::<Vec<MapDocument>>().await {
                            if list.len() < PAGE_SIZE {
                                has_more.set(false);
                            }
                            let mut current = (*maps).clone();
                            current.append(&mut list);
                            maps.set(current);
                            offset.set(*offset + PAGE_SIZE);
                        } else {
                            has_more.set(false);
                        }
                    }
                    is_loading.set(false);
                });
            },
            200, // debounce delay in milliseconds
        );
    }

    // Scroll event listener
    {
        let is_loading = is_loading.clone();
        let has_more = has_more.clone();

        use_effect(move || {
            let callback = Closure::wrap(Box::new(move || {
                if *is_loading || !*has_more {
                    return;
                }

                let window = window().unwrap();
                let document = window.document().unwrap();
                let body = document.body().unwrap();

                let scroll_top = window.scroll_y().unwrap_or(0.0);
                let window_height = window.inner_height().unwrap().as_f64().unwrap_or(0.0);
                let body_height = body.scroll_height() as f64;

                if scroll_top + window_height + 200.0 >= body_height {
                    // Trigger the debounced effect
                }
            }) as Box<dyn Fn()>);

            window()
                .unwrap()
                .add_event_listener_with_callback("scroll", callback.as_ref().unchecked_ref())
                .unwrap();

            callback.forget();

            || {}
        });
    }

    html! {
        <div id="catalog">
            { for (*maps).iter().map(move |m| html! {
                <MapAssetCard
                    asset={m.clone()}
                />
            }) }
            {
                if *is_loading {
                    html! { <div class="loading">{ "Loading more..." }</div> }
                } else {
                    html! {}
                }
            }
            {
                if !*has_more {
                    html! { <div class="end">{ "No more maps." }</div> }
                } else {
                    html! {}
                }
            }
        </div>
    }
}
