# System Requirements

There are several system requirements including fuelup, llvm, clang and postgres.

## Fuel system dependencies

Getting started with a Fuel indexer requires a single primary dependency from the Fuel ecosystem -- `fuelup`
- `fuelup` installs the Fuel toolchain from Fuel's official release channels, enabling you to easily keep the toolchain updated. For more info, take a look at the [`fuelup` repo](https://github.com/fuellabs/fuelup).

### Installation

To install `fuelup`

```bash
fuelup toolchain install latest
```

## Other system dependencies

### Ubuntu/Debian

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

### Arch

```bash
pacman -Syu --needed --noconfirm cmake \
    gcc openssl-1.0 pkgconf git clang llvm11 llvm11-libs postgresql-libs

export OPENSSL_LIB_DIR="/usr/lib/openssl-1.0";
export OPENSSL_INCLUDE_DIR="/usr/include/openssl-1.0"
```
