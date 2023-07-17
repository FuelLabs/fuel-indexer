use fuel_indexer::{ffi, Database, FtColumn, IndexEnv, IndexerResult};
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_lib::{
    fully_qualified_namespace, graphql::GraphQLSchema, manifest::Manifest, type_id,
};
use fuel_indexer_schema::db::manager::SchemaManager;
use wasmer::{imports, AsStoreMut, Cranelift, Instance, Module, Store};

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

async fn load_wasm_module(
    pool: IndexerConnectionPool,
    manifest: &Manifest,
) -> IndexerResult<(Instance, Store)> {
    let compiler = compiler();
    let mut store = Store::new(compiler);
    let module = Module::new(&store, SIMPLE_WASM_WASM)?;

    let env = wasmer::FunctionEnv::new(&mut store, IndexEnv::new(pool, manifest).await?);

    let mut import_object = imports! {};
    for (export_name, export) in ffi::get_exports(&mut store, &env) {
        import_object.define("env", &export_name, export.clone());
    }

    let instance = Instance::new(&mut store, &module, &import_object)?;

    Ok((instance, store))
}

#[tokio::test]
async fn test_schema_manager_generates_and_loads_schema_postgres() {
    if let Ok(mut current_dir) = std::env::current_dir() {
        if current_dir.ends_with("fuel-indexer-tests") {
            current_dir.pop();
            current_dir.pop();
        }

        if let Err(e) = std::env::set_current_dir(current_dir) {
            eprintln!("Failed to change directory: {}", e);
        }
    }
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

    let (instance, mut store) = load_wasm_module(pool.clone(), &manifest)
        .await
        .expect("Error creating WASM module");

    let version = ffi::get_version(&mut store.as_store_mut(), &instance)
        .expect("Could not get version");

    let mut db = Database::new(pool.clone(), &manifest).await;
    db.load_schema(version.clone())
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
        FtColumn::Address(Some(fuel_indexer_types::fuel::Address::from([0x04; 32]))),
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
