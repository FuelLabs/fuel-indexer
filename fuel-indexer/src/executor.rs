use crate::database::Database;
use crate::ffi;
use crate::{IndexerError, IndexerMessage, IndexerResult, Manifest, DEFAULT_PORT};
use async_std::sync::{Arc, Mutex};
use async_trait::async_trait;
use fuel_indexer_schema::{deserialize, serialize, BlockData};
use std::path::Path;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::{net::TcpStream, task::spawn_blocking};
use tracing::error;
use wasmer::{
    imports, Instance, LazyInit, Memory, Module, NativeFunc, RuntimeError, Store, WasmerEnv,
};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;

fn compiler() -> Cranelift {
    Cranelift::default()
}

#[async_trait]
pub trait Executor
where
    Self: Sized,
{
    fn from_file(index: &Path) -> IndexerResult<Self>;
    async fn handle_events(&mut self, blocks: Vec<BlockData>) -> IndexerResult<()>;
}

#[derive(Error, Debug)]
pub enum TxError {
    #[error("WASM Runtime Error {0:?}")]
    WasmRuntimeError(#[from] RuntimeError),
}

#[derive(WasmerEnv, Clone)]
pub struct IndexEnv {
    #[wasmer(export)]
    memory: LazyInit<Memory>,
    #[wasmer(export(name = "alloc_fn"))]
    alloc: LazyInit<NativeFunc<u32, u32>>,
    #[wasmer(export(name = "dealloc_fn"))]
    dealloc: LazyInit<NativeFunc<(u32, u32), ()>>,
    pub db: Arc<Mutex<Database>>,
}

pub struct NativeIndexExecutor {
    db: Arc<Mutex<Database>>,
    #[allow(dead_code)]
    manifest: Manifest,
    _process: tokio::process::Child,
    stream: TcpStream,
}

impl NativeIndexExecutor {
    pub async fn new(db_conn: &str, manifest: Manifest, path: String) -> IndexerResult<Self> {
        let db = Arc::new(Mutex::new(Database::new(db_conn).await.unwrap()));

        db.lock().await.load_schema_native(manifest.clone()).await?;

        let port = manifest.indexer_port.unwrap_or(DEFAULT_PORT);
        let process = tokio::process::Command::new(path)
            .arg(format!("--listen {}", port))
            .kill_on_drop(true)
            .spawn()?;

        let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;

        Ok(Self {
            db,
            manifest,
            _process: process,
            stream,
        })
    }
}

#[async_trait]
impl Executor for NativeIndexExecutor {
    fn from_file(_index: &Path) -> IndexerResult<Self> {
        unimplemented!()
    }

    async fn handle_events(&mut self, blocks: Vec<BlockData>) -> IndexerResult<()> {
        let mut buf = [0u8; 4096];

        let msg = serialize(&IndexerMessage::Blocks(blocks));

        self.stream.write_u64(msg.len() as u64).await?;
        self.stream.write_all(&msg).await?;

        loop {
            let size = self.stream.read_u64().await? as usize;

            if self.stream.read_exact(&mut buf[..size]).await? < size {
                return Err(IndexerError::HandlerError);
            }

            let object: IndexerMessage =
                deserialize(&buf[..size]).expect("Could not deserialize message from indexer!");

            match object {
                IndexerMessage::Object(_) | IndexerMessage::Blocks(_) => panic!("Not good"),
                IndexerMessage::GetObject(type_id, object_id) => {
                    let object = self.db.lock().await.get_object(type_id, object_id).await;
                    if let Some(obj) = object {
                        self.stream.write_u64(obj.len() as u64).await?;
                        self.stream.write_all(&obj).await?
                    } else {
                        self.stream.write_u64(0).await?;
                    }
                }
                IndexerMessage::PutObject(type_id, bytes, columns) => {
                    self.db
                        .lock()
                        .await
                        .put_object(type_id, columns, bytes)
                        .await;
                }
                IndexerMessage::Commit => break,
            }
        }
        Ok(())
    }
}

impl IndexEnv {
    pub async fn new(db_conn: String) -> IndexerResult<IndexEnv> {
        let db = Arc::new(Mutex::new(Database::new(&db_conn).await?));
        Ok(IndexEnv {
            memory: Default::default(),
            alloc: Default::default(),
            dealloc: Default::default(),
            db,
        })
    }
}

/// Responsible for loading a single indexer module, triggering events.
#[derive(Debug)]
pub struct WasmIndexExecutor {
    instance: Instance,
    _module: Module,
    _store: Store,
    db: Arc<Mutex<Database>>,
}

impl WasmIndexExecutor {
    pub async fn new(
        db_conn: String,
        _manifest: Manifest,
        wasm_bytes: impl AsRef<[u8]>,
    ) -> IndexerResult<Self> {

        let store = Store::new(&Universal::new(compiler()).engine());
        let module = Module::new(&store, &wasm_bytes)?;

        let mut import_object = imports! {};

        let mut env = IndexEnv::new(db_conn).await?;
        let exports = ffi::get_exports(&env, &store);
        import_object.register("env", exports);

        let instance = Instance::new(&module, &import_object)?;
        env.init_with_instance(&instance)?;
        env.db.lock().await.load_schema_wasm(&instance).await?;

        if !instance
            .exports
            .contains(ffi::MODULE_ENTRYPOINT.to_string())
        {
            return Err(IndexerError::MissingHandler);
        }

        Ok(WasmIndexExecutor {
            instance,
            _module: module,
            _store: store,
            db: env.db.clone(),
        })
    }
}

#[async_trait]
impl Executor for WasmIndexExecutor {
    /// Restore index from wasm file
    fn from_file(_index: &Path) -> IndexerResult<Self> {
        unimplemented!()
    }

    /// Trigger a WASM event handler, passing in a serialized event struct.
    async fn handle_events(&mut self, blocks: Vec<BlockData>) -> IndexerResult<()> {
        let bytes = serialize(&blocks);
        let arg = ffi::WasmArg::new(&self.instance, bytes)?;

        let fun = self
            .instance
            .exports
            .get_native_function::<(u32, u32), ()>(ffi::MODULE_ENTRYPOINT)?;

        self.db.lock().await.start_transaction().await?;

        let ptr = arg.get_ptr();
        let len = arg.get_len();

        let res = spawn_blocking(move || fun.call(ptr, len)).await?;

        if let Err(e) = res {
            error!("Indexer failed {e:?}");
            self.db.lock().await.revert_transaction().await?;
            return Err(IndexerError::RuntimeError(e));
        } else {
            self.db.lock().await.commit_transaction().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fuel_tx::Receipt;
    use fuels_abigen_macro::abigen;
    use fuels_core::{abi_encoder::ABIEncoder, Tokenizable};
    use sqlx::{Connection, Row};

    const MANIFEST: &str = include_str!("test_data/manifest.yaml");
    const BAD_MANIFEST: &str = include_str!("test_data/bad_manifest.yaml");
    const BAD_WASM_BYTES: &[u8] = include_bytes!("test_data/bad_simple_wasm.wasm");
    const WASM_BYTES: &[u8] = include_bytes!("test_data/simple_wasm.wasm");

    abigen!(MyContract, "fuel-indexer/src/test_data/contracts-abi.json");

    #[derive(Debug)]
    struct Thing1 {
        id: i64,
        account: String,
    }

    #[tokio::test]
    async fn test_postgres() {
        let database_url = "postgres://postgres:my-secret@127.0.0.1:5432";

        do_test(database_url).await;

        let mut conn = sqlx::PgConnection::connect(database_url)
            .await
            .expect("Database connection failed!");

        let row = sqlx::query("select id,account from test_namespace.thing1 where id = 1020;")
            .fetch_one(&mut conn)
            .await
            .expect("Database query failed");

        let id = row.get(0);
        let account = row.get(1);

        let data = Thing1 { id, account };

        assert_eq!(data.id, 1020);
        assert_eq!(
            data.account,
            "afafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafaf"
        );
    }

    #[tokio::test]
    async fn test_sqlite() {
        let workspace_root = env!("CARGO_MANIFEST_DIR");
        let database_url = format!("sqlite://{}/test.db", workspace_root);

        do_test(&database_url).await;

        let mut conn = sqlx::SqliteConnection::connect(&database_url)
            .await
            .expect("Database connection failed!");

        let row = sqlx::query("select id,account from thing1 where id = 1020;")
            .fetch_one(&mut conn)
            .await
            .expect("Database query failed");

        let id = row.get(0);
        let account = row.get(1);

        let data = Thing1 { id, account };

        assert_eq!(data.id, 1020);
        assert_eq!(
            data.account,
            "afafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafaf"
        );
    }

    async fn do_test(database_url: &str) {
        let manifest: Manifest = serde_yaml::from_str(MANIFEST).expect("Bad yaml file.");
        let bad_manifest: Manifest = serde_yaml::from_str(BAD_MANIFEST).expect("Bad yaml file.");

        let executor =
            WasmIndexExecutor::new(database_url.to_string(), bad_manifest, BAD_WASM_BYTES).await;
        match executor {
            Err(IndexerError::MissingHandler) => (),
            e => panic!("Expected missing handler error {:#?}", e),
        }

        let executor = WasmIndexExecutor::new(database_url.to_string(), manifest, WASM_BYTES).await;
        assert!(executor.is_ok());

        let mut executor = executor.unwrap();

        let evt1 = SomeEvent {
            id: 1020,
            account: [0xaf; 32],
        };
        let evt2 = AnotherEvent {
            id: 100,
            account: [0x5a; 32],
            hash: [0x43; 32],
        };

        let some_event = ABIEncoder::new()
            .encode(&[evt1.into_token()])
            .expect("Failed to encode");
        let another_event = ABIEncoder::new()
            .encode(&[evt2.into_token()])
            .expect("Failed to encode");

        let result = executor
            .handle_events(vec![BlockData {
                height: 0,
                transactions: vec![
                    vec![
                        Receipt::Call {
                            id: [0u8; 32].into(),
                            to: [0u8; 32].into(),
                            amount: 400,
                            asset_id: [0u8; 32].into(),
                            gas: 4,
                            param1: 2048508220,
                            param2: 0,
                            pc: 0,
                            is: 0,
                        },
                        Receipt::ReturnData {
                            id: [0u8; 32].into(),
                            ptr: 2342143,
                            len: some_event.len() as u64,
                            digest: [0u8; 32].into(),
                            data: some_event,
                            pc: 0,
                            is: 0,
                        },
                    ],
                    vec![
                        Receipt::Call {
                            id: [0u8; 32].into(),
                            to: [0u8; 32].into(),
                            amount: 400,
                            asset_id: [0u8; 32].into(),
                            gas: 4,
                            param1: 2379805026,
                            param2: 0,
                            pc: 0,
                            is: 0,
                        },
                        Receipt::ReturnData {
                            id: [0u8; 32].into(),
                            ptr: 2342143,
                            len: another_event.len() as u64,
                            digest: [0u8; 32].into(),
                            data: another_event,
                            pc: 0,
                            is: 0,
                        },
                    ],
                ],
            }])
            .await;
        assert!(result.is_ok());
    }
}
