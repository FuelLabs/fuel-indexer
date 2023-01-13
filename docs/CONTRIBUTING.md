# Contributing to Fuel Indexer

Thanks for your interest in contributing to Fuel Indexer! This document outlines some the conventions on building, running, and testing Fuel Indexer.

Fuel Indexer has many dependent repositories. If you need any help or mentoring getting started, understanding the codebase, or anything else, please ask on our [Discord](https://discord.gg/xfpK4Pe).

## Code Standards

We use an RFC process to maintain our code standards. They currently live in the RFC repo: <https://github.com/FuelLabs/rfcs/tree/master/text/code-standards>

## Building and setting up a development workspace

Fuel Core is mostly written in Rust, but includes components written in C++ (RocksDB).
We are currently using the latest Rust stable toolchain to build the project.
But for `rustfmt`, we use Rust nightly toolchain because it provides more code style features(you can check [`rustfmt.toml`](.rustfmt.toml)).

### Prerequisites

To build Fuel Core you'll need to at least have the following installed:

- `git` - version control
- [`rustup`](https://rustup.rs/) - Rust installer and toolchain manager
- [`clang`](http://releases.llvm.org/download.html) - Used to build system libraries (required for rocksdb).
- [`postgresql/libpq`](https://grpc.io/docs/protoc-installation/) - Used for Postgres backend.

See the [README.md](README.md#system-requirements) for platform specific setup steps.

### Getting the repository

> Future instructions assume you are in this repository

```sh
git clone https://github.com/FuelLabs/fuel-indexer
cd fuel-indexer
```

### Configuring your Rust toolchain

`rustup` is the official toolchain manager for Rust.

We use some additional components such as `clippy` and `rustfmt`, to install those:

```sh
rustup component add clippy
rustup component add rustfmt
```

Fuel Indexer also uses a few other tools installed via `cargo`

```sh
cargo install sqlx-cli
cargo install wasm-snip
```

### Building and testing

Fuel Indexer's two primary crates are `fuel-indexer` and `fuel-indexer-api-server`.

You can build Fuel Indexer:

```sh
cargo build -p fuel-indexer -p fuel-indexer-api-server
```

This command will run `cargo build` and also dump the latest schema into `/assets/` folder.

Linting is done using rustfmt and clippy, which are each separate commands:

```sh
cargo fmt --all --check
```

```sh
cargo clippy --all-features --all-targets -- -D warnings
```

The test suite follows the Rust cargo standards. The GraphQL service will be instantiated by
Tower and will emulate a server/client structure.

Testing is simply done using Cargo:

```sh
RUSTFLAGS='-D warnings' SQLX_OFFLINE=1 cargo test --locked --all-targets --all-features
```

#### Build Options

For optimal performance, we recommend using native builds. The generated binary will be optimized for your CPU and may contain specific instructions supported only in your hardware.

To build, run:

```sh
cargo build --release --bin fuel-indexer
```

The generated binary will be located in `./target/release/fuel-indexer`

### Build issues

- Due to dependencies on external components such as RocksDb, build times can be large without caching.
  We currently use [sccache](https://github.com/mozilla/sccache)

```sh
cargo build -p fuel-indexer --no-default-features
```

## Contribution flow

This is a rough outline of what a contributor's workflow looks like:

- Make sure what you want to contribute is already tracked as an issue.
    We may discuss the problem and solution in the issue.
  ⚠️ **DO NOT submit PRs that do not have an associated issue** ⚠️
- Create a Git branch from where you want to base your work.
  - Most work is usually branched off of `master`
  - Give your branch a name related to the work you're doing
- Write code, add test cases, and commit your work.
- Run tests and make sure all tests pass.
- Your commit message should be formatted as `[commit type]: [short commit blurb]`
  - Examples:
    - If you fixed a bug, your message is `fix: sqlite database locking issue`
    - If you added new functionality, your message would be `enhancement: i add
        something super cool`
    - If you just did a chore your message is: `chore: i did somthing not fun`
  - Keeping commit messages short and consistent helps users parse release
        notes
- Push up your branch to Github then (on the right hand side of the Github UI):
  - Assign yourself as the owner of the PR
  - Add any and all necessary labels to your PR
  - Link the issue your PR solves, to your PR
- If you are part of the FuelLabs Github org, please open a PR from the repository itself.
- Otherwise, push your changes to a branch in your fork of the repository and submit a pull request.
  - Make sure mention the issue, which is created at step 1, in the commit message.
- Your PR will be reviewed and some changes may be requested.
  - Once you've made changes, your PR must be re-reviewed and approved.
  - If the PR becomes out of date, you can use GitHub's 'update branch' button.
  - If there are conflicts, you can merge and resolve them locally. Then push to your PR branch.
    - Any changes to the branch will require a re-review.
- Our CI (Github Actions) automatically tests all authorized pull requests.
- Use Github to merge the PR once approved.

### Commit categories
- `bug`: If fixing broken functionality
- `enhancement`: If adding new functionality
- `chore`: If finishing valuable work (that's no fun!)
- `testing`: If only updating/writing tests
- `docs`: If just updating docs
- `feat`: If adding a non-trivial new feature
- There will be categories not covered in this doc - use your best judgement!

Thanks for your contributions!

## Finding something to work on

For beginners, we have prepared many suitable tasks for you. Checkout our [Good First Issues](https://github.com/FuelLabs/fuel-indexer/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) for a list.

If you are planning something that relates to multiple components or changes current behaviors, make sure to open an issue to discuss with us before continuing.
