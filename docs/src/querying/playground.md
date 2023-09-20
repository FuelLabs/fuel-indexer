# GraphQL Playground

The Fuel indexer's GraphQL Playground is an interactive, in-browser GraphQL IDE that allows developers to easily explore and test the indexer's GraphQL API server. You can read more about the GraphQL playground in general [here](https://github.com/graphql/graphql-playground).

Every public indexer can access the GraphQL playground of the Fuel indexer node on which the given indexer runs, so users and devs can get to querying their data right away.

## Usage

To use the GraphQL playground to explor your indices, simply [start your indexer service](../getting-started/indexer-service-infrastructure.md) - then open the following URL in your browser - where `namespace` and `identifier` correspond to the namespace and identifier of the index that you'd like to explore.

```bash
http://localhost:29987/api/playground/:namespace/:identifier
```
