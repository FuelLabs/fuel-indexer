# System Requirements

There are a few system requirements that you'll need to get started with a Fuel indexer.

## Other system dependencies

Other system dependencies related to compilation, tooling, and SQL backends, include:

### Ubuntu/Debian

```bash
apt update && apt install -y \
    cmake \
    pkg-config \
    git \
    gcc \
    build-essential \
    clang \
    libclang-dev \
    llvm \
    libpq-dev
```

| Dependency | Required For |
| --------------- | --------------- |
| cmake | Manages the build process in an operating system and in a compiler-independent manner |
| pkg-config | Language-agnostic helper tool used when compiling applications and libraries |
| git | Version control system |
| gcc | Compiler tools required to build various Fuel indexer crates |
| clang/libclang-dev | Compiler tools required to build various Fuel indexer crates on Unix-like OSes |
| llvm | Required for building Fuel indexer crate dependencies |
| libpq-dev | Set of library function helping facilitate interaction with the PostgreSQL backend |

### MacOS

```bash
brew update && brew install \
    cmake \
    llvm \
    libpq \
    postgresql
```

| Dependency | Required For |
| --------------- | --------------- |
| cmake | Manages the build process in an operating system and in a compiler-independent manner |
| llvm| Compiler infrastructure for building Fuel indexer crate dependencies |
| libpq | Postgres C API library |
| postgresql | Installs the command line console (psql) as well as a PostgreSQL server locally  |

### Arch

```bash
pacman -Syu --needed --noconfirm \
    cmake \
    gcc \
    pkgconf \
    git \
    clang \
    llvm11 \
    llvm11-libs \
    postgresql-libs
```

| Dependency | Required For |
| --------------- | --------------- |
| cmake | Manages the build process in an operating system and in a compiler-independent manner |
| git | Version control system |
| gcc | Compiler tools required to build various Fuel indexer crates |
| llvm11 | Compiler infrastructure for building Fuel indexer crate dependencies |
| llvm11-libs | Compiler infrastructure libs for building Fuel indexer crate dependencies |
| pkgconf | System for configuring build dependency information |
| postgresql-libs | Provides the essential shared libraries for any PostgreSQL client program or interface |
| clang | Compiler required to build various Fuel indexer crates Unix-like OSes |
