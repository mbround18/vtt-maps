CREATE TABLE IF NOT EXISTS map_download_events (
    id BIGSERIAL PRIMARY KEY,
    map_id TEXT NOT NULL,
    client_ip TEXT,
    user_agent TEXT,
    downloaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_map_download_events_map_id ON map_download_events (map_id);

CREATE INDEX IF NOT EXISTS idx_map_download_events_downloaded_at ON map_download_events (downloaded_at);