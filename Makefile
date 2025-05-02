.PHONY: all setup lint serve-meilisearch serve-backend serve-frontend serve clean setup_meilisearch

# Efficient Makefile for building and managing Rust and WebAssembly projects
# Following the approach in https://markentier.tech/posts/2022/01/speedy-rust-builds-under-wsl2/
# to improve Rust build performance under WSL2.

SOURCE_DIR := $(PWD)
PROJECT_NAME := $(notdir $(SOURCE_DIR))
BUILD_DIR := ~/tmp/$(PROJECT_NAME)
DIST_DIR := $(SOURCE_DIR)/dist
INDEX_FILE := packages/gh-pagify/index.html

# MeiliSearch configuration
MEILI_MASTER_KEY ?= masterKey
MEILI_URL ?= http://127.0.0.1:7700
MEILI_KEY ?= $(MEILI_MASTER_KEY)

all: build

setup:
	@mkdir -p $(BUILD_DIR)
	@mkdir -p $(DIST_DIR)
	@cargo install trunk cargo-watch
	@docker compose pull meilisearch
	@echo "Setup completed successfully."

lint:
	@npx prettier --write .
	@cargo fmt
	@echo "Linting completed successfully."


serve-meilisearch:
	@docker compose up -d meilisearch
	@echo "Waiting for MeiliSearch to be ready..."
	@sleep 2
	@make setup_meilisearch

serve-backend:
	@echo "Starting backend server..."
	@cargo watch --env MEILI_MASTER_KEY=$(MEILI_MASTER_KEY) \
                 --env MEILI_URL=$(MEILI_URL) \
                 --env MEILI_KEY=$(MEILI_KEY) \
                 --env RUST_LOG=debug \
                 -x 'run --bin actix-backend'

serve-frontend:
	@echo "Starting frontend server..."
	@cd packages/yew-frontend && RUST_LOG=info trunk watch --dist $(DIST_DIR)

serve: lint serve-meilisearch
	@(trap 'kill 0' SIGINT; make serve-frontend & make serve-backend & wait)


clean:
	@cargo clean --target-dir $(BUILD_DIR)
	@rm -rf $(DIST_DIR)
	@echo "Clean up completed successfully."


setup_meilisearch:
	@docker compose up -d meilisearch
	@sleep 2
	@curl -s -X POST "$(MEILI_URL)/indexes" \
		--header "Content-Type: application/json" \
		--header "Authorization: Bearer $(MEILI_MASTER_KEY)" \
		--data "{\"uid\":\"maps\",\"primaryKey\":\"id\"}"
	@echo "maps index created or already exists."
