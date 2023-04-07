# GraphQL Playground

- GraphQL Playground is an interactive, in-browser GraphQL IDE that allows developers to easily explore and test GraphQL APIs. 
You can read more about it [here](https://github.com/graphql/graphql-playground)
- Indexers come shipped with their own GraphQL playground, so you can get to querying your data right away. 
## Usage

- To use the playground, with your indexer running simply go to this url in your browser, where `namespace` and `identifier` are those found in your manifest.

```bash
http://localhost:29987/api/playground/{namespace}/{identifier}
```

- Note: On initial page render, the playground throws a 500. Don't worry about this, you should still be able to query. 

