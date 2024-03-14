# Following this tutorial: https://markentier.tech/posts/2022/01/speedy-rust-builds-under-wsl2/
# This makes developing on windows significantly easier for rust projects!!

SOURCE_DIR = $(PWD)
# `notdir` returns the part after the last `/`
# so if the source was "/some/nested/project", only "project" remains
BUILD_DIR  = ~/tmp/$(notdir $(SOURCE_DIR))

lint:
	npx prettier --write .
	cargo fmt

build:
	cargo build

compile:
	trunk build packages/gh-pagify/index.html \
    			--dist packages/gh-pagify/dist \
    && cargo run --release --bin ssr-catalog

release:
	cargo run --release --bin ssr-catalog


serve: compile
	npx http-server packages/gh-pagify/dist

thumbs:
	cargo run --bin thumbnail-generator "./maps"

dev: 
	cargo run --bin ssr-catalog
