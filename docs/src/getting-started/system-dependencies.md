## System Requirements

There are several system requirements including forc, llvm, clang and postgres.

### Installing `forc`

- On all architectures, install the following Fuel crates via `cargo`:

```bash
cargo install forc fuel-core
```

- `forc` is the crate that holds the Sway language and Fuel's equivalent of `cargo`
- `fuel-core` is the crate that contains the Fuel node software and execution

### Ubuntu

```bash
apt update
apt install -y cmake pkg-config libssl-dev git \
    gcc build-essential git clang libclang-dev llvm libpq-dev
```

### MacOS

```bash
brew update
brew install openssl cmake llvm libpq postgresql
```

### Debian

```bash
apt update
apt install -y cmake pkg-config libssl-dev git \
    gcc build-essential git clang libclang-dev llvm libpq-dev
```

### Arch

```bash
pacman -Syu --needed --noconfirm cmake \
    gcc openssl-1.0 pkgconf git clang llvm11 llvm11-libs postgresql-libs

export OPENSSL_LIB_DIR="/usr/lib/openssl-1.0";
export OPENSSL_INCLUDE_DIR="/usr/include/openssl-1.0"
```
