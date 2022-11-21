use fuel_indexer::{ffi, Database, FtColumn, IndexEnv, IndexerResult};
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_lib::config::DatabaseConfig;
use fuel_indexer_schema::{
    db::manager::SchemaManager,
    utils::{inject_native_entities_into_schema, schema_version},
};
use fuel_indexer_tests::fixtures::test_sqlite_db_path;
use fuel_types::Address;
use wasmer::{imports, Instance, Module, Store, WasmerEnv};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;

fn compiler() -> Cranelift {
    Cranelift::default()
}

// TODO: sqlite and postgres now....
const GRAPHQL_SCHEMA: &str = include_str!("./../assets/simple_wasm.graphql");
const SIMPLE_WASM_WASM: &[u8] = include_bytes!("./../assets/simple_wasm.wasm");
const THING1_TYPE: u64 = 0xA21A262A00405632;
const TEST_COLUMNS: [(&str, i32, &str); 10] = [
    ("thing2", 0, "id"),
    ("thing2", 1, "account"),
    ("thing2", 2, "hash"),
    ("thing2", 3, "object"),
    ("thing1", 0, "id"),
    ("thing1", 1, "account"),
    ("thing1", 2, "object"),
    ("indexmetadataentity", 0, "id"),
    ("indexmetadataentity", 1, "time"),
    ("indexmetadataentity", 2, "object"),
];

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
    let database_url = DatabaseConfig::default().to_string();
    generate_schema_then_load_schema_from_wasm_module(&database_url).await;
}

#[tokio::test]
async fn test_schema_manager_generates_and_loads_schema_sqlite() {
    generate_schema_then_load_schema_from_wasm_module(&test_sqlite_db_path()).await;
}

async fn generate_schema_then_load_schema_from_wasm_module(database_url: &str) {
    let manager = SchemaManager::new(database_url)
        .await
        .expect("Could not create SchemaManager");

    // SchemaManager.build calls inject_native_entities_into_schema so since we're using
    // `version` later in this test we need to manually call `inject_native_entities_into_schema` here
    let schema = inject_native_entities_into_schema(GRAPHQL_SCHEMA);

    let result = manager.new_schema("test_namespace", GRAPHQL_SCHEMA).await;
    assert!(result.is_ok());

    let pool = IndexerConnectionPool::connect(database_url)
        .await
        .expect("Connection pool error");

    let version = schema_version(&schema);
    let mut conn = pool
        .acquire()
        .await
        .expect("Failed to acquire indexer connection");
    let results = queries::columns_get_schema(&mut conn, "test_namespace", &version)
        .await
        .expect("Metadata query failed");

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

    db.load_schema_wasm(&instance)
        .await
        .expect("Could not load db schema");

    assert_eq!(db.namespace, "test_namespace");
    assert_eq!(db.version, version);

    for column in TEST_COLUMNS.iter() {
        assert!(db.schema.contains_key(column.0));
    }

    let object_id = 4;
    let columns = vec![
        FtColumn::ID(object_id),
        FtColumn::Address(Address::from([0x04; 32])),
    ];
    let bytes = vec![0u8, 1u8, 2u8, 3u8];
    db.start_transaction()
        .await
        .expect("Start transaction failed");
    db.put_object(THING1_TYPE, columns, bytes.clone()).await;

    let obj = db.get_object(THING1_TYPE, object_id).await;
    assert!(obj.is_some());
    let obj = obj.expect("Failed to get object from database");

    assert_eq!(obj, bytes);

    assert_eq!(db.get_object(THING1_TYPE, 90).await, None);
}
