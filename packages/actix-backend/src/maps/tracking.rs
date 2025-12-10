use actix_web::{
    HttpRequest, HttpResponse,
    error::{ErrorBadRequest, ErrorInternalServerError},
    web,
};
use anyhow::Context;
use chrono::{Duration, Utc};
use diesel::{dsl::count_star, prelude::*};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, warn};

use crate::schema::map_download_events;
use crate::utils::db::DbPool;

#[derive(Debug, Clone)]
pub struct DownloadEventMetadata {
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
}

impl DownloadEventMetadata {
    pub fn from_request(req: &HttpRequest) -> Self {
        let connection_info = req.connection_info();
        let client_ip = connection_info.realip_remote_addr().map(str::to_string);
        let user_agent = req
            .headers()
            .get(actix_web::http::header::USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);

        Self {
            client_ip,
            user_agent,
        }
    }
}

#[derive(Deserialize)]
pub struct TrackDownloadPayload {
    pub map_id: String,
}

#[derive(Serialize)]
pub struct DownloadMetricsResponse {
    pub downloads: Vec<MapDownloadMetrics>,
}

#[derive(Serialize)]
pub struct MapDownloadMetrics {
    pub map_id: String,
    pub total_downloads: i64,
    pub last_30_days: i64,
}

#[derive(Insertable)]
#[diesel(table_name = map_download_events)]
struct NewMapDownloadEvent {
    map_id: String,
    client_ip: Option<String>,
    user_agent: Option<String>,
}

pub async fn record_download_event(
    pool: web::Data<DbPool>,
    map_id: String,
    metadata: DownloadEventMetadata,
) -> Result<(), actix_web::Error> {
    let pool = pool.clone();
    let result = web::block(move || -> Result<(), anyhow::Error> {
        let mut conn = pool
            .get()
            .context("Failed to get DB connection for logging download event")?;

        diesel::insert_into(map_download_events::table)
            .values(&NewMapDownloadEvent {
                map_id,
                client_ip: metadata.client_ip,
                user_agent: metadata.user_agent,
            })
            .execute(&mut conn)
            .context("Failed to insert map_download_events row")?;

        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => {
            error!("Download tracking insert failed: {}", e);
            Err(ErrorInternalServerError("Failed to record download event"))
        }
        Err(e) => {
            error!("Download tracking thread failed: {}", e);
            Err(ErrorInternalServerError("Failed to record download event"))
        }
    }
}

pub async fn track_download(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    payload: web::Json<TrackDownloadPayload>,
) -> Result<HttpResponse, actix_web::Error> {
    if payload.map_id.trim().is_empty() {
        return Err(ErrorBadRequest("map_id is required"));
    }

    let metadata = DownloadEventMetadata::from_request(&req);
    // Record the event before responding; failures are logged but do not affect the response
    if let Err(e) = record_download_event(pool, payload.map_id.trim().to_string(), metadata).await {
        warn!(
            "Failed to record tracking event for {}: {}",
            payload.map_id, e
        );
    }

    Ok(HttpResponse::Accepted().json(serde_json::json!({
        "status": "accepted",
        "map_id": payload.map_id,
        "queued": true,
    })))
}

pub async fn download_metrics(pool: web::Data<DbPool>) -> Result<HttpResponse, actix_web::Error> {
    let pool = pool.clone();
    let result = web::block(move || -> Result<Vec<MapDownloadMetrics>, anyhow::Error> {
        let mut conn = pool
            .get()
            .context("Failed to get DB connection for metrics query")?;

        use crate::schema::map_download_events::dsl::{
            downloaded_at as downloaded_at_col, map_download_events as events, map_id as map_id_col,
        };

        let totals = events
            .group_by(map_id_col)
            .select((map_id_col, count_star()))
            .load::<(String, i64)>(&mut conn)
            .context("Failed to load total download metrics")?;

        let thirty_days_ago = (Utc::now() - Duration::days(30)).naive_utc();
        let trending_rows = events
            .filter(downloaded_at_col.ge(thirty_days_ago))
            .group_by(map_id_col)
            .select((map_id_col, count_star()))
            .load::<(String, i64)>(&mut conn)
            .context("Failed to load 30-day download metrics")?;

        let mut last_30_map: HashMap<String, i64> = trending_rows.into_iter().collect();

        let metrics = totals
            .into_iter()
            .map(|(map_identifier, total_downloads)| MapDownloadMetrics {
                last_30_days: last_30_map.remove(&map_identifier).unwrap_or(0),
                map_id: map_identifier,
                total_downloads,
            })
            .collect();

        Ok(metrics)
    })
    .await;

    let mut aggregates = match result {
        Ok(Ok(rows)) => rows,
        Ok(Err(e)) => {
            error!("Metrics query failed: {}", e);
            return Err(ErrorInternalServerError("Failed to load download metrics"));
        }
        Err(e) => {
            error!("Metrics query thread failed: {}", e);
            return Err(ErrorInternalServerError("Failed to load download metrics"));
        }
    };

    // Sort by total downloads desc by default to keep response stable
    aggregates.sort_by(|a, b| b.total_downloads.cmp(&a.total_downloads));

    Ok(HttpResponse::Ok().json(DownloadMetricsResponse {
        downloads: aggregates,
    }))
}
