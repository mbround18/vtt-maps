# Copilot Instructions

## Styling

- Use SCSS only, under `packages/yew-frontend/src/styles`; new styles must be wired through `index.scss`.

## Architecture & Data Flow

- Workspace members live in `packages/`; `actix-backend` serves APIs/static assets, `yew-frontend` builds the SPA shell via Trunk, `shared` hosts types/utilities consumed by both.
- Map content lives in `maps/**` with previews under `assets/thumbnails/**`; backend boot (`main.rs`) runs `setup_folders` then `rebuild_maps_init` to refresh Meilisearch’s `maps` index from every `.dd2vtt` file.
- `shared::types::map_document::MapDocument` is the contract between API responses and frontend components; keep additions backwards compatible.
- Static HTML responses flow through `wrappers::seo::SeoMetadata` which injects metadata (default vs `/maps/:id` specific) before `file_service` serves the SPA fallback.

## Backend Workflows

- Entry point: `packages/actix-backend/src/main.rs`; routes are grouped by scope (`/api/maps`, `/api/docs`, `/health`, `/admin`). Requests under `/api/maps` hit handlers in `maps/*` which all resolve documents through `clients::meilisearch` and `shared` types.
- Map rebuild lifecycle (`maps/rebuild.rs`) enforces a `.map_rebuild_lock.json` in `assets/thumbnails`; admin-only endpoints (`POST /api/maps/rebuild`, `DELETE /api/maps/rebuild/clear`) are protected by `hooks::admin_auth::AdminAuth` and rely on the `.admin-token` file or `ADMIN_TOKEN` env.
- Containers can set `REPO_DIR`/`REPO_REF`; `utils::setup`/`repo` clone or update the upstream repo before indexing. Use `scripts/check-rebuild-status.sh` to inspect stale locks inside containers.
- Markdown endpoints (`/api/docs/readme|license` and `/api/maps/content/{id}`) render through `docs::serve_markdown_file`, which converts Markdown to HTML via `utils::markdown`.
- Static files: `services::file_service` serves `/dist` with SPA fallback; `/assets/thumbnails` is exposed via `actix_files::Files`. Keep `DIST_DIR` in sync if you relocate build artifacts.
- Security middleware: `hooks::cors`, `hooks::identity`, `hooks::security`, and `wrappers::seo::SeoMetadata` are added in that order—do not reorder without understanding header expectations.

## Frontend Practices

- App root (`src/main.rs`) wraps `Header` + `BrowserRouter`; routes are defined in `pages/mod.rs` and must remain in sync with server SEO handling (`/maps/:id` is treated specially).
- HTTP access goes through `api::context::ApiEndpoint`; reuse these builders so query params (`limit`, `offset`) stay consistent with backend defaults.
- Pages use `use_effect_*` + `spawn_local` for async fetches. `MapDetail` maintains an explicit `LoadingState` state machine and preloads the tiled PNG via a hidden `<img>`; respect that flow when extending.
- Components pull shared data (`MapDocument`, casing helpers) from `packages/shared`; mutations should happen in hooks before render (components like `MapAssetCard` are still struct components, others are function components).
- Only add CSS in SCSS partials under `src/styles/**` and import them via `index.scss`; pages/components already have matching partials (e.g., `styles/pages`, `styles/components`).

## Shared Crate & Assets

- `shared/src/types` defines `MapDocument`, `MapResolution`, `MapReference`, and `DD2VTTFile`. `maps/rebuild.rs` converts `DD2VTTFile` → `MapReference` → Meilisearch docs; keep that pipeline stable.
- `shared::utils::root_dir` respects `REPO_DIR` and ensures folders exist; always use it (or helpers like `maps_dir`, `assets_dir`) instead of `std::env::current_dir`.
- Thumbnail generation uses `DD2VTTFile::export_thumbnail_file` which downscales by 16×; if you need different sizes, add new helpers rather than inlining image logic.
- String helpers (`utils::casing`) are already used by frontend components; reuse them instead of duplicating formatting logic.

## Tooling & Commands

- `Makefile` drives workflows: `make setup` installs `trunk`/`cargo-watch` and pulls Meilisearch, `make build` runs Trunk + `cargo build --release`, `make serve` runs frontend/watch + backend + Meilisearch in parallel.
- Frontend builds use `trunk build --release --dist dist`; dev server is `trunk watch --dist dist` from `packages/yew-frontend` (Makefile wraps this with `serve-frontend`).
- Backend live-reloads via `cargo watch -x 'run --bin actix-backend'` (Makefile target `serve-backend`). Production builds write to `BUILD_DIR` (defaults to `~/tmp/<project>` unless `CI` is set).
- Meilisearch is started with `make serve-meilisearch`; `setup_meilisearch` seeds the `maps` index via REST. Keep `MEILI_URL`, `MEILI_KEY`, and `MEILI_MASTER_KEY` aligned across services.

## Coding Practices

- Keep changes concise: avoid extra comments/explanations unless requested; stick to Rust/Yew idioms and reuse existing abstractions before inventing new ones.
- Tests should focus on the directly affected crate using its existing framework (`actix_web::test` for backend middleware, Yew hooks/components for frontend, Rust unit tests for `shared`).
- Follow DRY/KISS/YAGNI and SOLID principles already in use; prefer composing small modules (e.g., new map handlers belong in `maps/*` with exports via `mod.rs`).
- Release builds use size optimizations (`panic=abort`, `opt-level=z`, LTO); avoid patterns that rely on unwinding or large static data blobs.
