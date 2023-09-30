# Stage 1: Build
FROM --platform=$BUILDPLATFORM tonistiigi/xx AS xx
FROM --platform=$BUILDPLATFORM rust:1.72.1 AS chef

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

RUN xx-cargo chef cook --release -p hello_indexer_native --recipe-path recipe.json
COPY . .
RUN xx-cargo build --release -p hello_indexer_native \
    && xx-verify ./target/$(xx-cargo --print-target-triple)/release/hello_indexer_native \
    && mv ./target/$(xx-cargo --print-target-triple)/release/hello_indexer_native ./target/release/hello_indexer_native \
    && mv ./target/$(xx-cargo --print-target-triple)/release/hello_indexer_native.d ./target/release/hello_indexer_native.d

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

COPY --from=builder /build/target/release/hello_indexer_native  .
COPY --from=builder /build/target/release/hello_indexer_native.d .
COPY --from=builder /build/examples/hello-world-native/hello-indexer-native .

RUN echo "namespace: fuellabs\n\
fuel_client: ~\n\
graphql_schema: schema/hello_indexer_native.graphql\n\
abi: fuel-indexer-test-abi.json\n\
start_block: ~\n\
end_block: ~\n\
contract_id: fuel1h97g4w7wrwv78ad5xquxs3ecl30xstuqgzsw4uvpakg7yyh8d86spsken2\n\
identifier: hello_indexer_native\n\
module: native\n\
resumable: true" > hello_indexer_native.manifest.yaml