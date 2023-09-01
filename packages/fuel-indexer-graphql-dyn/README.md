# fuel-indexer-graphql-dyn

GraphQL Dyn is a library for building executable dynamic GraphQL schemas.

Dyn has two main parts: the `store::Store` and the `schema::DynSchemaBuilder`. The schema builder is responsible for creating executable `async_graphql_dynamic::dynamic::Schema` instances from a dynamically defined `schema::DynSchemaType`. Built schema executes GraphQL queries by fetching data from the store. The store is responsible for communicating with the underlying data source.

## Usage

Users of Dyn will need three things to get started:

```rs
/// A `store::Store` implementation.
#[async_trait]
impl Store for MyStore {
    fn r#type(&self) -> &StoreType { ... }
    //
    async fn obj_get(&self, id: &Id) -> R<O<Obj>> { ... }
    async fn obj_get_many(&self, ids: &[Id]) -> R<Vec<O<Obj>>> { ... }
    async fn assoc_get(&self, key: &Key, ...) -> R<Vec<O<Assoc>>> { ... }
    async fn assoc_count(&self, key: &Key, ...) -> R<u64> { ... }
    async fn assoc_range(&self, key: &Key, ...) -> R<Vec<Assoc>> { ... }
    async fn assoc_time_range(&self, key: &Key, ...) -> R<Vec<Assoc>> { ... }
}

/// A `store::StoreType` constructor.
fn new_store_type(config: &MyConfig) -> StoreType {
    let mut store_type = StoreTypeBuilder::default();
    ...
    store_type.finish()
}

/// A `schema::DynSchemaType` constructor.
fn new_schema_type(config: &MyConfig, store_type: &StoreType) -> DynSchemaType {
    let mut schema_type = DynSchemaTypeBuilder::new(store_type);
    ...
    schema_type.finish()
}
```

With these implemented, you can create a dynamic schema and execute queries:

```rs
let store_type = new_store_type(&my_config)?;
let schema_type = new_schema_type(&my_config, &store_type)?;
let store = MyStore::new(&store_type);

let builder = DynSchemaBuilder::new(&schema_type, Arc::new(Mutex::new(store)));
let schema = builder.finish()?;

let response = schema.execute(Request::new(r#"
    query {
        chain(id: "Chain:0") {
            name
            ...
            blocks(first: 10) {
                nodes {
                    number
                    ...
                    transactions(first: 10) {
                        nodes {
                            index
                            ...
                        }
                        pageInfo {
                            hasNextPage
                            hasPreviousPage
                            startCursor
                            endCursor
                        }
                    }
                }
            }
        }
    }
"#)).await;
```
