use crate::components::map_asset_card::MapAssetCard;
use shared::types::map_document::MapDocument;

use crate::api::api::ApiEndpoint;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_hooks::prelude::*;

#[function_component(Catalog)]
pub fn catalog() -> Html {
    let maps = use_state(Vec::<MapDocument>::new);
    let is_loading = use_state(|| true);

    {
        let maps = maps.clone();
        let is_loading = is_loading.clone();

        use_effect_once(move || {
            spawn_local(async move {
                if let Ok(response) = {
                    ApiEndpoint::GetAllMaps {
                        limit: None,
                        offset: None,
                    }
                }
                .request()
                .send()
                .await
                {
                    if let Ok(list) = response.json::<Vec<MapDocument>>().await {
                        maps.set(list);
                    }
                }

                is_loading.set(false);
            });
            || {}
        });
    }

    html! {
        <div id="catalog">
            { for (*maps).iter().map(|m| html! {
                <MapAssetCard asset={m.clone()} />
            }) }
            {
                if *is_loading {
                    html! { <div class="loading">{ "Loading..." }</div> }
                } else {
                    html! {}
                }
            }
        </div>
    }
}
