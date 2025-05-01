.PHONY: lint build compile release serve serve-meilisearch serve-backend serve-frontend clean all docker-build docker-up docker-dev setup_meilisearch

# Efficient Makefile for building and managing Rust and WebAssembly projects
# Following the approach in https://markentier.tech/posts/2022/01/speedy-rust-builds-under-wsl2/
# to improve Rust build performance under WSL2.

SOURCE_DIR := $(PWD)
PROJECT_NAME := $(notdir $(SOURCE_DIR))
BUILD_DIR := ~/tmp/$(PROJECT_NAME)
DIST_DIRS := ./dist packages/gh-pagify/dist
INDEX_FILE := packages/gh-pagify/index.html

# MeiliSearch configuration
MEILI_MASTER_KEY ?= masterKey
MEILI_URL ?= http://127.0.0.1:7700
MEILI_KEY ?= $(MEILI_MASTER_KEY)

all: build

lint:
	@npx prettier --write .
	@cargo fmt
	@echo "Linting completed successfully."

build:
	@cargo build --target-dir $(BUILD_DIR)
	@echo "Build completed successfully."

compile: build
	@cd packages/yew-frontend && RUST_LOG=info trunk build --release --dist ./dist --minify 
	@echo "Compilation and SSR completed successfully."

serve-meilisearch:
	@docker compose up -d meilisearch
	@echo "Waiting for MeiliSearch to be ready..."
	@sleep 2
	@make setup_meilisearch

serve-backend:
	@echo "Starting backend server..."
	@MEILI_MASTER_KEY=$(MEILI_MASTER_KEY) MEILI_URL=$(MEILI_URL) MEILI_KEY=$(MEILI_KEY) \
		RUST_LOG=debug cargo run --bin actix-backend

serve-frontend:
	@echo "Starting frontend server..."
	@cd packages/yew-frontend && RUST_LOG=info trunk serve --port 3000 --address 0.0.0.0 

serve: lint serve-meilisearch
	@(trap 'kill 0' SIGINT; make serve-frontend & make serve-backend & wait)


clean:
	@cargo clean --target-dir $(BUILD_DIR)
	@rm -rf $(DIST_DIRS)
	@echo "Clean up completed successfully."


docker-build: compile
	@docker compose build

docker-up:
	@docker compose up


docker-dev: docker-build docker-up

setup_meilisearch:
	@docker compose up -d meilisearch
	@sleep 2
	@curl -s -X POST "$(MEILI_URL)/indexes" \
		--header "Content-Type: application/json" \
		--header "Authorization: Bearer $(MEILI_MASTER_KEY)" \
		--data "{\"uid\":\"maps\",\"primaryKey\":\"id\"}"
	@echo "maps index created or already exists."
