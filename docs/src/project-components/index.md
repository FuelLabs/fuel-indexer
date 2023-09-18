<!-- markdownlint-disable-file MD024 -->
# A Fuel Indexer Project

## Use Cases

The Fuel indexer project can currently be used in a number of different ways:

- as tooling to interact with indexers
- as a standalone service
- as a part of a larger Fuel project, alongside other components of the Fuel ecosystem (e.g. [Sway smart contracts](https://fuellabs.github.io/sway))

We'll describe these three different use cases below.

### As tooling to interact with indexers

The Fuel indexer provides functionality to make it easy to build and compile abitrary indexers by using the [`forc index`](../forc-index/index.md) CLI tool. Using `forc index`, users can create, build, deploy, and remove indexers, as well as authenticate against a running indexer service, and check the status of running indexers.

#### Example

Create, deploy, and check the status of a new indexer.

```bash
forc index new fuel && \
    cd fuel && forc index deploy --url http://indexer.fuel.network && \
        forc index status --url http://indexer.fuel.network --auth $MY_TOKEN
```

### As a standalone service

You can also start the Fuel indexer as a standalone service that connects to a Fuel node in order to monitor the Fuel blockchain for new blocks and transactions. To do so, run the requisite database migrations, adjust the configuration to connect to a Fuel node, and start the service.

#### Example

Create, deploy, and check the status of a new indexer.

```bash
fuel-indexer run \
    --fuel-node-host beta-4.fuel.network \
    --fuel-node-port 80 \
    --run-migrations \
    --accept-sql-queries \
    --replace-indexer
```

### As part of a Fuel project

Finally, you can run the Fuel indexer as part of a project that uses other components of the Fuel ecosystem, such as Sway. The convention for a Fuel project layout including an indexer is as follows:

```text
.
├── contracts
│   └── hello-contract
│       ├── Forc.toml
│       └── src
│           └── main.sw
├── frontend
│   └── index.html
└── indexer
    └── hello-indexer
        ├── Cargo.toml
        ├── hello_indexer.manifest.yaml
        ├── schema
        │   └── hello_indexer.schema.graphql
        └── src
            └── lib.rs
```

---

## An Indexer Project at a Glance

Every Fuel indexer project requires three components:

- a [Manifest](./manifest.md) describing how the indexer should work
- a [Schema](./schema.md) containing data models for the data that is to be indexed
- a [Module](./module.md) which contains the logic for how data coming from the FuelVM should be saved into an index
