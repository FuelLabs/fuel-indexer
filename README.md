# Fuel Indexer

[![build](https://github.com/FuelLabs/fuel-indexer/actions/workflows/ci.yml/badge.svg)](https://github.com/FuelLabs/fuel-indexer/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/fuel-indexer?label=latest)](https://crates.io/crates/fuel-indexer)
[![docs](https://docs.rs/fuel-indexer/badge.svg)](https://docs.rs/fuel-indexer/)
[![discord](https://img.shields.io/badge/chat%20on-discord-orange?&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/xfpK4Pe)

Fuel Indexer uses WASM to index transactions and state within a fuel network, allowing for high-performance read-only
access to the blockchain for advanced dApp use-cases.

# Building

##### System Requirements

There are several system requirements including llvm, clang and postgres.


###### MacOS
```bash
brew update
brew install openssl cmake llvm libpq postgresql
```

###### Debian
```bash
apt update
apt install -y cmake pkg-config libssl-dev git gcc build-essential git clang libclang-dev llvm libpq-dev
```

###### Arch
```bash 
pacman -Syu --needed --noconfirm cmake gcc openssl-1.0 pkgconf git clang llvm11 llvm11-libs postgresql-libs
export OPENSSL_LIB_DIR="/usr/lib/openssl-1.0";
export OPENSSL_INCLUDE_DIR="/usr/include/openssl-1.0"
```

