# Contributing to Fuel Indexer

Thanks for your interest in contributing to Fuel Indexer! This document outlines some the conventions on building, running, and testing Fuel Indexer.

Fuel Indexer has many dependent repositories. If you need any help or mentoring getting started, understanding the codebase, or anything else, please ask on our [Discord](https://discord.gg/xfpK4Pe).

## Code Standards

- [ ] If you've added a new function, method, class or abstraction, please include `rustdoc` comments for the new code so others can better understand the change.
- [ ] If your change is non-trivial and testable, please try to include at least one happy path test to ensure that your change works.
  - "Trivial" changes would be changes to docs, comments, or small style/syntactic changes

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
cargo build -p fuel-indexer -p fuel-indexer-api-server --release --locked
```

Linting is done using `rustfmt` and `clippy`, which are each separate commands:

```sh
cargo fmt --all --check
```

```sh
cargo clippy --all-features --all-targets -- -D warnings
```

The test suite follows the Rust cargo standards.

Testing is simply done using Cargo:

```sh
RUSTFLAGS='-D warnings' cargo test --locked --all-targets --all-features
```

To run `trybuild` tests.

```sh
RUSTFLAGS='-D warnings' TRYBUILD=overwrite cargo test --locked --all-targets --all-features
```

## Contribution flow

This is a rough outline of what a contributor's workflow looks like:

- Make sure what you want to contribute is already tracked as an issue.
    We may discuss the problem and solution in the issue.
  ⚠️ **DO NOT submit PRs that do not have an associated issue** ⚠️
- Create a Git branch from where you want to base your work.
  - Most work is usually branched off of `develop`
  - Give your branch a name related to the work you're doing
    - The convention for branch naming is usually `1234/short-description`, where `1234` is the number of the associated issue.
- Write code, add test cases, and commit your work.
- Run tests and make sure all tests pass.
- Your commit message should be formatted as `[commit type]: [short commit blurb]`
  - Examples:
    - If you fixed a bug, your message is `fix: database locking issue`
    - If you added new functionality, your message would be `enhancement: i added something super cool`
    - If you just did a chore your message is: `chore: i helped do the chores`
  - Keeping commit messages short and consistent helps users parse release notes
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
