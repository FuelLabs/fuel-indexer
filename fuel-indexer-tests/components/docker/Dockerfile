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

ENV SQLX_OFFLINE=true

FROM installer AS builder
WORKDIR /build/
COPY . .
RUN rustup component add rustfmt
RUN rustup target add wasm32-unknown-unknown
RUN cargo build --release -p composable-indexer -p fuel-indexer
RUN cd tests/e2e/composable-indexer/composable-indexer-lib && cargo build --release


FROM ubuntu:22.04
WORKDIR /root/
RUN DEBIAN_FRONTEND=noninteractive apt-get update -y \
    && apt-get install -y --no-install-recommends \
    tzdata \
    ca-certificates \
    libpq-dev \
    wget \
    && apt-get autoremove -y \
    && apt-get clean -y

# https://stackoverflow.com/a/72633324
RUN wget http://archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2_amd64.deb
RUN dpkg -i libssl1.1_1.1.1f-1ubuntu2_amd64.deb

COPY --from=builder /build/target/release/fuel-node .
COPY --from=builder /build/target/release/fuel-node.d .

COPY --from=builder /build/target/release/web-api .
COPY --from=builder /build/target/release/web-api.d .

COPY --from=builder /build/target/release/fuel-indexer .
COPY --from=builder /build/target/release/fuel-indexer.d .

COPY --from=builder /build/fuel-indexer-tests/e2e/composable-indexer/composable-indexer-lib/contracts/ping/out/debug/ping.bin .

COPY --from=builder /build/fuel-indexer-tests/e2e/composable-indexer/ .

COPY --from=builder /build/target/wasm32-unknown-unknown/release/composable_indexer_lib.wasm ./composable-indexer-lib/
