.PHONY: lint build compile release serve thumbnails ssr_catalog clean all docker-build

# Efficient Makefile for building and managing Rust and WebAssembly projects
# Following the approach in https://markentier.tech/posts/2022/01/speedy-rust-builds-under-wsl2/
# to improve Rust build performance under WSL2.

SOURCE_DIR := $(PWD)
PROJECT_NAME := $(notdir $(SOURCE_DIR))
BUILD_DIR := ~/tmp/$(PROJECT_NAME)
DIST_DIRS := ./dist packages/gh-pagify/dist
INDEX_FILE := packages/gh-pagify/index.html

all: build

lint:
	@npx prettier --write .
	@cargo fmt
	@echo "Linting completed successfully."

build: ssr_catalog
	@cargo build --target-dir $(BUILD_DIR)
	@echo "Build completed successfully."

ssr_catalog: thumbnails
	@cargo run --bin ssr_catalog --release
	@echo "SSR catalog generation completed."

compile: build
	@trunk build $(INDEX_FILE) --dist ./dist --minify
	@echo "Compilation and SSR completed successfully."

serve: ssr_catalog
	@trunk serve $(INDEX_FILE) --no-autoreload

thumbnails:
	@cargo run --bin thumbnail-generator "./maps"
	@echo "Thumbnail generation completed."

clean:
	@cargo clean --target-dir $(BUILD_DIR)
	@rm -rf $(DIST_DIRS)
	@echo "Clean up completed successfully."


docker-build: compile
	@docker compose build

