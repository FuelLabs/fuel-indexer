# Stage 1: Build
FROM --platform=$BUILDPLATFORM tonistiigi/xx AS xx
FROM --platform=$BUILDPLATFORM rust:1.71.1 AS chef

ARG TARGETPLATFORM
RUN cargo install cargo-chef
WORKDIR /build/

COPY --from=xx / /

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
    pkg-config
   
RUN xx-apt-get update && \
    xx-apt-get install -y binutils g++ libc6-dev && \
    apt-get clean

FROM chef AS planner
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: build binaries
FROM chef AS builder
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
COPY --from=planner /build/recipe.json recipe.json

ENV SQLX_OFFLINE=true

RUN xx-cargo chef cook --release -p fuel-indexer -p fuel-indexer-api-server --features fuel-core-lib --recipe-path recipe.json
COPY . .
RUN xx-cargo build --release -p fuel-indexer -p fuel-indexer-api-server --features fuel-core-lib \
    && xx-verify ./target/$(xx-cargo --print-target-triple)/release/fuel-indexer \
    && xx-verify ./target/$(xx-cargo --print-target-triple)/release/fuel-indexer-api-server \
    && mv ./target/$(xx-cargo --print-target-triple)/release/fuel-indexer ./target/release/fuel-indexer \
    && mv ./target/$(xx-cargo --print-target-triple)/release/fuel-indexer-api-server ./target/release/fuel-indexer-api-server \
    && mv ./target/$(xx-cargo --print-target-triple)/release/fuel-indexer.d ./target/release/fuel-indexer.d \
    && mv ./target/$(xx-cargo --print-target-triple)/release/fuel-indexer-api-server.d ./target/release/fuel-indexer-api-server.d

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
