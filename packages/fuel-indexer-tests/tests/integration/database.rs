use fuel_indexer::{ffi, Database, FtColumn, IndexEnv, IndexerResult};
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_lib::{
    fully_qualified_namespace, graphql::GraphQLSchema, manifest::Manifest, type_id,
};
use fuel_indexer_schema::db::manager::SchemaManager;
use fuels::prelude::Address;
use wasmer::{imports, Instance, Module, Store, WasmerEnv};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;

fn compiler() -> Cranelift {
    Cranelift::default()
}

const SIMPLE_WASM_MANIFEST: &str =
    include_str!("./../../components/indices/simple-wasm/simple_wasm.yaml");
const SIMPLE_WASM_GRAPHQL_SCHEMA: &str =
    include_str!("./../../components/indices/simple-wasm/schema/simple_wasm.graphql");
const SIMPLE_WASM_WASM: &[u8] =
    include_bytes!("./../../components/indices/simple-wasm/simple_wasm.wasm");
const TEST_COLUMNS: [(&str, i32, &str); 11] = [
    ("thing2", 0, "id"),
    ("thing2", 1, "account"),
    ("thing2", 2, "hash"),
    ("thing2", 3, "object"),
    ("indexmetadataentity", 0, "id"),
    ("indexmetadataentity", 1, "time"),
    ("indexmetadataentity", 2, "block_height"),
    ("indexmetadataentity", 3, "object"),
    ("thing1", 0, "id"),
    ("thing1", 1, "account"),
    ("thing1", 2, "object"),
];
const TEST_NAMESPACE: &str = "test_namespace";
const TEST_INDENTIFIER: &str = "simple_wasm_executor";

async fn load_wasm_module(database_url: &str) -> IndexerResult<Instance> {
    let compiler = compiler();
    let store = Store::new(&Universal::new(compiler).engine());
    let module = Module::new(&store, SIMPLE_WASM_WASM)?;

    let mut import_object = imports! {};
    let mut env = IndexEnv::new(database_url.to_string()).await?;

    let exports = ffi::get_exports(&env, &store);
    import_object.register("env", exports);

    let instance = Instance::new(&module, &import_object)?;
    env.init_with_instance(&instance)?;

    Ok(instance)
}

#[tokio::test]
async fn test_schema_manager_generates_and_loads_schema_postgres() {
    let database_url = "postgres://postgres:my-secret@localhost:5432";
    generate_schema_then_load_schema_from_wasm_module(database_url).await;
}

async fn generate_schema_then_load_schema_from_wasm_module(database_url: &str) {
    let pool = IndexerConnectionPool::connect(database_url)
        .await
        .expect("Connection pool error");

    let mut conn = pool
        .acquire()
        .await
        .expect("Failed to acquire indexer connection");
    queries::run_migration(&mut conn)
        .await
        .expect("Failed to run migrations");

    let manager = SchemaManager::new(pool.clone());

    let manifest = Manifest::try_from(SIMPLE_WASM_MANIFEST).unwrap();
    let schema = GraphQLSchema::new(SIMPLE_WASM_GRAPHQL_SCHEMA.to_owned());

    let result = manager
        .new_schema(
            "test_namespace",
            "simple_wasm_executor",
            SIMPLE_WASM_GRAPHQL_SCHEMA,
            manifest.execution_source(),
            &mut conn,
        )
        .await;

    assert!(result.is_ok());

    let version = schema.version().to_owned();
    let results = queries::columns_get_schema(
        &mut conn,
        "test_namespace",
        "simple_wasm_executor",
        &version,
    )
    .await
    .expect("Metadata query failed");

    assert_eq!(results.len(), TEST_COLUMNS.len());

    for (index, result) in results.into_iter().enumerate() {
        assert_eq!(result.table_name, TEST_COLUMNS[index].0);
        assert_eq!(result.column_position, TEST_COLUMNS[index].1);
        assert_eq!(result.column_name, TEST_COLUMNS[index].2);
    }

    let instance = load_wasm_module(database_url)
        .await
        .expect("Error creating WASM module");

    let mut db = Database::new(database_url)
        .await
        .expect("Failed to create database object.");

    db.load_schema(&manifest, Some(&instance))
        .await
        .expect("Could not load db schema");

    assert_eq!(db.namespace, "test_namespace");
    assert_eq!(db.version, version);

    for column in TEST_COLUMNS.iter() {
        let key = format!("{}_{}.{}", TEST_NAMESPACE, TEST_INDENTIFIER, column.0);
        assert!(db.schema.contains_key(&key));
    }

    let object_id = 4;
    let columns = vec![
        FtColumn::ID(Some(object_id)),
        FtColumn::Address(Some(Address::from([0x04; 32]))),
    ];

    let thing1_ty_id = type_id(
        &fully_qualified_namespace(TEST_NAMESPACE, TEST_INDENTIFIER),
        "Thing1",
    );
    let bytes = vec![0u8, 1u8, 2u8, 3u8];
    db.start_transaction()
        .await
        .expect("Start transaction failed");
    db.put_object(thing1_ty_id, columns, bytes.clone()).await;

    db.commit_transaction()
        .await
        .expect("commit transaction failed");

    db.start_transaction()
        .await
        .expect("Start transaction failed");

    let obj = db.get_object(thing1_ty_id, object_id).await;

    assert!(obj.is_some());
    let obj = obj.expect("Failed to get object from database");

    assert_eq!(obj, bytes);

    assert_eq!(db.get_object(thing1_ty_id, 90).await, None);
}
