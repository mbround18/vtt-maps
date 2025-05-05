ARG RUST_VERSION=1.86
ARG DEBIAN_VERSION=bookworm
########################
# Base Setup
########################
FROM rust:${RUST_VERSION}-slim AS base

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates pkg-config libssl-dev build-essential \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install --locked cargo-chef sccache

ENV RUSTC_WRAPPER=sccache

ENV SCCACHE_DIR=/sccache

###############
# Trunk Setup
###############
FROM debian:${DEBIAN_VERSION}-slim AS trunk

# Install required tools
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        curl \
        ca-certificates \
        jq \
        tar && \
    rm -rf /var/lib/apt/lists/*

# Set environment variables
ENV REPO="https://api.github.com/repos/trunk-rs/trunk/releases/latest" \
    ARCH="x86_64-unknown-linux-gnu"

# Fetch and install the latest Trunk binary
RUN --mount=type=cache,target=/var/cache/apt \
    set -eux; \
    LATEST_VERSION=$(curl -sSL "${REPO}" | jq -r '.tag_name'); \
    curl -sSL "https://github.com/trunk-rs/trunk/releases/download/${LATEST_VERSION}/trunk-${ARCH}.tar.gz" | \
    tar -xz -C /usr/local/bin trunk


########################
# Dependency Planner
########################
FROM base AS planner

WORKDIR /app

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

########################
# Builder
########################
FROM base AS builder
WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/sccache \
    cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN rustup target add wasm32-unknown-unknown
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/sccache \
    cargo build --release

COPY --from=trunk /usr/local/bin/trunk /usr/local/bin/trunk

RUN cd ./packages/yew-frontend && trunk build --release \
    --public-url / \
    --dist /app/dist

########################
# Final Image
########################
FROM debian:${DEBIAN_VERSION}-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/dist ./dist
COPY --from=builder /app/target/release/actix-backend ./server
COPY ./assets ./assets

ENV DIST_DIR="/app/dist"

RUN adduser --disabled-password --gecos "" appuser \
    && chown -R appuser:appuser /app \
    && chmod +x /app/server \
    && mkdir -p /data \
    && chown -R appuser:appuser /data

USER appuser

CMD ["/app/server"]