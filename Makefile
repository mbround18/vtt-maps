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

wsl.compile: wsl.sync
	cd $(BUILD_DIR) \
		&& make compile

wsl.serve: wsl.compile
	rsync -avh $(BUILD_DIR)/packages/gh-pagify/ $(SOURCE_DIR)/packages/gh-pagify/ \
		--exclude src \
		--exclude Cargo.toml
	npx http-server packages/gh-pagify/dist --proxy http://127.0.0.1:8080? --push-state --cors


wsl.build: wsl.sync
	cd $(BUILD_DIR) && make build
	make wsl.reverse-sync

wsl.run: wsl.sync
	cd $(BUILD_DIR) && cargo run

wsl.test: wsl.sync
	cd $(BUILD_DIR) && cargo test

wsl.sync:
	mkdir -p $(BUILD_DIR)
	rsync -avh --delete-before $(SOURCE_DIR)/ $(BUILD_DIR)/ \
		--exclude .git \
		--exclude target \
		--exclude .fingerprint \
		--exclude build \
		--exclude incremental \
		--exclude deps	\
		--exclude .idea

wsl.reverse-sync:
	rsync -av $(BUILD_DIR)/target/debug/ $(SOURCE_DIR)/target/debug/ \
		--exclude .git \
		--exclude target \
		--exclude .fingerprint \
		--exclude build \
		--exclude incremental \
		--exclude deps	\
		--exclude .idea \
		--exclude packages

wsl.clean:
	rm -rf $(BUILD_DIR)/target
	rm -rf $(BUILD_DIR)/**/dist

wsl.clean-all:
	rm -rf $(BUILD_DIR)

wsl.clippy: wsl.sync
	cd $(BUILD_DIR) \
		&& cargo clippy

wsl.thumbs: wsl.build
	cd $(BUILD_DIR) \
		&& cargo run --bin thumbnail-generator "./"
	make wsl.reverse-sync

wsl.dev: wsl.sync
	cd $(BUILD_DIR) \
		&& cargo run --bin ssr-catalog


