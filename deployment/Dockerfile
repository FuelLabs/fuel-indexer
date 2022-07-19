# Stage 1: Build
FROM lukemathwalker/cargo-chef:latest-rust-1.61 as chef

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

FROM chef as planner
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: build binaries
FROM chef as builder
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release -p fuel-indexer --recipe-path recipe.json
COPY . .
RUN cargo build --release -p fuel-indexer --all-targets

# Stage 3: Run
FROM ubuntu:22.04 as run

WORKDIR /root/

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends \
    ca-certificates \
    libpq-dev \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y

COPY --from=builder /build/target/release/fuel-indexer .
COPY --from=builder /build/target/release/fuel-indexer.d .

ARG GRAPHQL_API_PORT=29987

EXPOSE $GRAPHQL_API_PORT

# hadolint ignore=DL3025
ENTRYPOINT ["./fuel-executor"]
