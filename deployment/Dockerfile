# Stage 1: Build
FROM lukemathwalker/cargo-chef:latest-rust-1.61 as chef

WORKDIR /build/

# hadolint ignore=DL3008
RUN apt-get update && \
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
ENV GRAPHQL_API_PORT=29987

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Run
FROM ubuntu:22.04 as run

WORKDIR /root/

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y


COPY target/release/fuel-indexer .
COPY target/release/fuel-indexer.d .

COPY target/release/api_server .
COPY target/release/api_server.d .

EXPOSE $GRAPHQL_API_PORT

# # https://stackoverflow.com/a/44671685
# # https://stackoverflow.com/a/40454758
# # hadolint ignore=DL3025
CMD exec ./fuel-indexer
