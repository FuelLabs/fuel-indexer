FROM lukemathwalker/cargo-chef:latest-rust-1.62 AS installer

WORKDIR /app

RUN apt-get update -y && \
    apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    clang \
    cmake \
    gcc \
    git \
    libclang-dev \
    libpq-dev \
    lld \
    llvm \
    pkg-config \
    && apt-get clean

FROM installer AS builder

WORKDIR /app

COPY . .
RUN rustup component add rustfmt
RUN rustup target add wasm32-unknown-unknown
RUN cargo build --release -p composable-indexer -p fuel-indexer 

RUN chmod +x tests/e2e/composable-indexer/make-test-manifest.bash
RUN bash tests/e2e/composable-indexer/make-test-manifest.bash
