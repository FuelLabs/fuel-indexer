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

RUN xx-cargo chef cook --release -p greetings_native_indexer --recipe-path recipe.json
COPY . .
RUN xx-cargo build --release -p greetings_native_indexer \
    && xx-verify ./target/$(xx-cargo --print-target-triple)/release/greetings_native_indexer \
    && mv ./target/$(xx-cargo --print-target-triple)/release/greetings_native_indexer ./target/release/greetings_native_indexer \
    && mv ./target/$(xx-cargo --print-target-triple)/release/greetings_native_indexer.d ./target/release/greetings_native_indexer.d

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

COPY --from=builder /build/target/release/greetings_native_indexer  .
COPY --from=builder /build/target/release/greetings_native_indexer.d .
COPY --from=builder /build/examples/greetings-native/greetings-native-indexer .
COPY --from=builder /build/examples/greetings/contracts/greeting/out/debug/greeting-abi.json .

# Since unlike WASM indexers, native indexers can't be deployed, we need to
# write the specific asset paths to the manifest, as well as the manifest itself.
#
# We only have to do this because the example is contained within the project worksapce (and it's
# manifest asset paths are relative to that workspace root). You wouldn't need to do this otherwise.
RUN echo "namespace: fuellabs\n\
fuel_client: ~\n\
graphql_schema: schema/greetings_native_indexer.schema.graphql\n\
abi: greetings-abi.son\n\
start_block: ~\n\
end_block: ~\n\
contract_id: fuel1q6sj2srt0u40jdqg2lvvnspyuhse9rs2s2fv9nmv0hqjcrdc7sqsfpwv9x\n\
identifier: greetings_native\n\
module: native\n\
resumable: true" > greetings_native_indexer.manifest.yaml