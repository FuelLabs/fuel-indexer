FROM lukemathwalker/cargo-chef:latest-rust-1.62 AS installer
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
WORKDIR /build/
COPY . .
RUN rustup component add rustfmt
RUN cargo build --release -p composable-indexer -p fuel-indexer


FROM ubuntu:22.04
WORKDIR /usr/local/composable-indexer
RUN DEBIAN_FRONTEND=noninteractive apt-get update -y \
    && apt-get install -y --no-install-recommends \
    tzdata \
    ca-certificates \
    libpq-dev \
    && apt-get autoremove -y \
    && apt-get clean -y

COPY --from=builder /build/target/release/fuel-node .
COPY --from=builder /build/target/release/fuel-node.d .

COPY --from=builder /build/target/release/web-api .
COPY --from=builder /build/target/release/web-api.d .

COPY --from=builder /build/target/release/fuel-indexer .
COPY --from=builder /build/target/release/fuel-indexer.d .

COPY --from=builder /build/tests/e2e/composable-indexer/composable-indexer-lib/contracts/ping/out/debug/ping.bin .

COPY --from=builder /build/tests/e2e/composable-indexer .


RUN bash make-test-manifest.bash
