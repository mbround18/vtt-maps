use crate::components::map_asset_card::MapAssetCard;
use serde::Deserialize;
use shared::types::map_document::MapDocument;
use std::cmp::Ordering;
use std::collections::HashMap;

use crate::api::context::ApiEndpoint;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlSelectElement;
use yew::prelude::*;

#[derive(Debug, Deserialize, Clone)]
struct DownloadMetricsResponse {
    downloads: Vec<DownloadMetricEntry>,
}

#[derive(Debug, Deserialize, Clone)]
struct DownloadMetricEntry {
    map_id: String,
    total_downloads: i64,
    last_30_days: i64,
}

#[derive(Debug, Clone)]
struct DownloadMetricStats {
    total_downloads: i64,
    last_30_days: i64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortMode {
    Popularity,
    Alphabetical,
    Trending,
}

impl SortMode {
    fn from_value(value: &str) -> Self {
        match value {
            "alphabetical" => SortMode::Alphabetical,
            "trending" => SortMode::Trending,
            _ => SortMode::Popularity,
        }
    }

    fn as_value(&self) -> &'static str {
        match self {
            SortMode::Popularity => "popularity",
            SortMode::Alphabetical => "alphabetical",
            SortMode::Trending => "trending",
        }
    }
}

#[function_component(Catalog)]
pub fn catalog() -> Html {
    let maps = use_state(Vec::<MapDocument>::new);
    let is_loading = use_state(|| true);
    let download_metrics = use_state(HashMap::<String, DownloadMetricStats>::new);
    let sort_mode = use_state(|| SortMode::Popularity);

    {
        let maps = maps.clone();
        let is_loading = is_loading.clone();

        use_effect(|| {
            spawn_local(async move {
                if let Ok(response) = {
                    ApiEndpoint::AllMaps {
                        limit: None,
                        offset: None,
                    }
                }
                .request()
                .send()
                .await
                    && let Ok(list) = response.json::<Vec<MapDocument>>().await
                {
                    maps.set(list);
                }

                is_loading.set(false);
            });
            || {}
        });
    }

    {
        let download_metrics = download_metrics.clone();
        use_effect(|| {
            spawn_local(async move {
                if let Ok(response) = ApiEndpoint::DownloadMetrics.request().send().await
                    && let Ok(metrics) = response.json::<DownloadMetricsResponse>().await
                {
                    let mut counts = HashMap::new();
                    for entry in metrics.downloads {
                        counts.insert(
                            entry.map_id,
                            DownloadMetricStats {
                                total_downloads: entry.total_downloads,
                                last_30_days: entry.last_30_days,
                            },
                        );
                    }
                    download_metrics.set(counts);
                }
            });
            || {}
        });
    }

    let on_sort_change = {
        let sort_mode = sort_mode.clone();
        Callback::from(move |event: Event| {
            if let Some(target) = event.target()
                && let Ok(select) = target.dyn_into::<HtmlSelectElement>()
            {
                sort_mode.set(SortMode::from_value(&select.value()));
            }
        })
    };

    let sorted_maps = {
        let mut data = (*maps).clone();
        let metrics_map = (*download_metrics).clone();
        let sort_mode_value = *sort_mode;

        data.sort_by(|a, b| match sort_mode_value {
            SortMode::Alphabetical => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortMode::Trending => {
                let a_trend = metrics_map.get(&a.id).map(|m| m.last_30_days).unwrap_or(0);
                let b_trend = metrics_map.get(&b.id).map(|m| m.last_30_days).unwrap_or(0);
                match b_trend.cmp(&a_trend) {
                    Ordering::Equal => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    other => other,
                }
            }
            SortMode::Popularity => {
                let a_total = metrics_map
                    .get(&a.id)
                    .map(|m| m.total_downloads)
                    .unwrap_or(0);
                let b_total = metrics_map
                    .get(&b.id)
                    .map(|m| m.total_downloads)
                    .unwrap_or(0);
                match b_total.cmp(&a_total) {
                    Ordering::Equal => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    other => other,
                }
            }
        });

        data
    };

    let metrics_handle = download_metrics.clone();
    let current_sort_value = (*sort_mode).as_value().to_string();

    html! {
        <section class="catalog-shell">
            <div class="catalog-controls">
                <label for="catalog-sort">{"Sort by"}</label>
                <select
                    id="catalog-sort"
                    onchange={on_sort_change}
                    value={current_sort_value.clone()}
                >
                    <option value="popularity">{"Popularity"}</option>
                    <option value="trending">{"Trending (30 days)"}</option>
                    <option value="alphabetical">{"Alphabetical"}</option>
                </select>
            </div>
            <div id="catalog">
            {
                for sorted_maps.iter().map(move |m| {
                    let counts = metrics_handle.get(&m.id);
                    let total = counts.map(|c| c.total_downloads);
                    let trend = counts.map(|c| c.last_30_days);
                    html! {
                        <MapAssetCard
                            asset={m.clone()}
                            download_count={total}
                            download_last_30d={trend}
                        />
                    }
                })
            }
            {
                if *is_loading {
                    html! { <div class="loading">{ "Loading..." }</div> }
                } else {
                    html! {}
                }
            }
            </div>
        </section>
    }
}
