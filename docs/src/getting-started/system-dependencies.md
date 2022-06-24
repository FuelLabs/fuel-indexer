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
    gcc build-essential clang libclang-dev llvm libpq-dev
```
| Dependency | Required For |
| --------------- | --------------- |
| cmake | Required for building Fuel Indexer crate dependencies |
| pkg-config | Required for building Fuel Indexer crate dependencies |
| libssl-dev | Required for building Fuel Indexer crate dependencies |
| git | Required for building Fuel Indexer crate dependencies |
| gcc | Required for building Fuel Indexer crate dependencies |
| clang | Required for building Fuel Indexer crate dependencies |
| llvm | Required for building Fuel Indexer crate dependencies |
| libpq-dev | Row 3 Column 2 |

### MacOS

```bash
brew update
brew install cmake llvm libpq postgresql
```

| Dependency | Required For |
| --------------- | --------------- |
| cmake | Required for building Fuel Indexer crate dependencies |
| llvm| Required for building Fuel Indexer crate dependencies |
| libq | Required for building Fuel Indexer crate dependencies |
| postgresql | Required for building Fuel Indexer crate dependencies |


### Arch

```bash
pacman -Syu --needed --noconfirm cmake \
    gcc pkgconf git clang llvm11 llvm11-libs postgresql-libs
```

| Dependency | Required For |
| --------------- | --------------- |
| cmake | Required for building Fuel Indexer crate dependencies |
| git | Required for building Fuel Indexer crate dependencies |
| gcc | Required for building Fuel Indexer crate dependencies |
| llvm11 | Required for building Fuel Indexer crate dependencies |
| llvm11-libs | Required for building Fuel Indexer crate dependencies |
| pkgconf | Required for building Fuel Indexer crate dependencies |
| postgresql-libs | Required for building Fuel Indexer crate dependencies |
| clang | Required for building Fuel Indexer crate dependencies |
