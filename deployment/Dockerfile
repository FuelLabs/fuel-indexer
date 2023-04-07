# Stage 1: Build
FROM lukemathwalker/cargo-chef:latest-rust-1.68.1 AS chef

WORKDIR /build/

# hadolint ignore=DL3008
RUN apt-get update -y && \
    apt-get install -y --no-install-recommends \
    build-essential \
    clang \
    cmake \
    gcc \
    git \
    libclang-dev \
    lld \
    llvm \
    pkg-config \
    && apt-get clean

FROM chef AS planner
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: build binaries
FROM chef AS builder
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
COPY --from=planner /build/recipe.json recipe.json

ENV SQLX_OFFLINE=true

RUN cargo chef cook --release -p fuel-indexer --features fuel-core-lib --recipe-path recipe.json
RUN cargo chef cook --release -p fuel-indexer-api-server --features fuel-core-lib --recipe-path recipe.json
COPY . .
RUN cargo build --release -p fuel-indexer -p fuel-indexer-api-server --features fuel-core-lib --all-targets

# Stage 3: Run
FROM ubuntu:22.04 AS run

WORKDIR /root/

RUN DEBIAN_FRONTEND=noninteractive apt-get update -y \
    && apt-get install -y --no-install-recommends \
    tzdata \
    ca-certificates \
    libpq-dev \
    wget \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y

COPY --from=builder /build/target/release/fuel-indexer .
COPY --from=builder /build/target/release/fuel-indexer.d .

COPY --from=builder /build/target/release/fuel-indexer-api-server .
COPY --from=builder /build/target/release/fuel-indexer-api-server.d .
