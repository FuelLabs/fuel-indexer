# Fuel Indexer

![Fuel Logo](./docs/src/img/fuel.png "Fuel Logo")

[![build](https://github.com/FuelLabs/fuel-indexer/actions/workflows/ci.yml/badge.svg)](https://github.com/FuelLabs/fuel-indexer/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/fuel-indexer?label=latest)](https://crates.io/crates/fuel-indexer)
[![docs](https://docs.rs/fuel-indexer/badge.svg)](https://docs.rs/fuel-indexer/)
[![discord](https://img.shields.io/badge/chat%20on-discord-orange?&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/xfpK4Pe)

The Fuel Indexer is a standalone binary that can be used to index various components of the blockchain. These indexable components include blocks, transactions, receipts, and state within a Fuel network, allowing for high-performance read-only access to the blockchain for advanced dApp use-cases.

## Usage

### Clone repository

```bash
git clone git@github.com:FuelLabs/fuel-indexer.git
```

### Run migrations

```bash
DATABASE_URL=postgres://postgres@localhost sh scripts/run_migrations.local.sh
```
