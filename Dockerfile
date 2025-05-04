FROM ubuntu:latest AS trunk

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        curl \
        ca-certificates \
        jq \
        tar && \
    rm -rf /var/lib/apt/lists/*

ENV TRUNK_REPO="https://api.github.com/repos/trunk-rs/trunk/releases/latest" \
    ARCH="x86_64-unknown-linux-gnu"

RUN --mount=type=cache,target=/var/cache/apt \
    set -eux; \
    LATEST_VERSION=$(curl -sSL $TRUNK_REPO | jq -r '.tag_name'); \
    curl -sSL "https://github.com/trunk-rs/trunk/releases/download/${LATEST_VERSION}/trunk-${ARCH}.tar.gz" | \
    tar -xz -C /usr/local/bin trunk

FROM rust:1.86 AS builder

WORKDIR /usr/src/app

COPY --from=trunk /usr/local/bin/trunk /usr/local/bin/trunk

COPY . .

RUN rustup target add wasm32-unknown-unknown

RUN cargo build --release
RUN cd ./packages/yew-frontend && trunk build --release \
    --public-url / \
    --dist /usr/src/app/dist

FROM ubuntu:latest

WORKDIR /usr/src/app

ENV DIST_DIR=/usr/src/app/dist \
    REPO_PATH=/data \
    REPO_REF="main"

RUN mkdir -p $REPO_PATH && \
    mkdir -p $DIST_DIR

COPY --from=builder /usr/src/app/dist /usr/src/app/dist
COPY --from=builder /usr/src/app/target/release/actix-backend /usr/src/server

COPY ./assets /usr/src/app/assets

CMD ["/usr/src/server"]
