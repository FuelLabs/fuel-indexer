use async_trait::async_trait;
use crate::database::Database;
use crate::ffi;
use crate::handler::{Handle, ReceiptEvent};
use crate::{IndexerError, IndexerResult, Manifest};
use fuel_tx::Receipt;
use std::collections::HashMap;
use std::path::Path;
use tokio::task::spawn_blocking;
use async_std::sync::{Arc, Mutex};
use thiserror::Error;
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
    async fn trigger_event(
        &self,
        event_name: ReceiptEvent,
        bytes: Vec<Vec<u8>>,
        receipt: Option<Receipt>,
    ) -> IndexerResult<()>;
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
    handles: Vec<Handle>,
    db: Arc<Mutex<Database>>,
    #[allow(dead_code)]
    manifest: Manifest,
}

impl NativeIndexExecutor {
    pub async fn new(db_conn: &str, manifest: Manifest, handles: Vec<Handle>) -> IndexerResult<Self> {
        let db = Arc::new(Mutex::new(Database::new(db_conn).await.unwrap()));

        db.lock().await.load_schema_native(manifest.clone()).await?;

        Ok(Self {
            handles,
            db,
            manifest,
        })
    }
}

#[async_trait]
impl Executor for NativeIndexExecutor {
    fn from_file(_index: &Path) -> IndexerResult<Self> {
        unimplemented!()
    }

    async fn trigger_event(
        &self,
        _event: ReceiptEvent,
        _bytes: Vec<Vec<u8>>,
        receipt: Option<Receipt>,
    ) -> IndexerResult<()> {
        for handle in self.handles.iter() {
            self.db.lock().await.start_transaction().await?;

            if let Some(receipt) = receipt.clone() {
                let data = receipt.data().unwrap().to_vec();
                if let Some(result) = handle(receipt) {
                    match result {
                        Ok(result) => {
                            self.db.lock().await.put_object(result.0, result.1, data).await;

                            self.db
                                .lock()
                                .await
                                .commit_transaction().await?;
                        }
                        Err(e) => {
                            error!("Indexer failed {e:?}");
                            self.db
                                .lock()
                                .await
                                .revert_transaction().await?;
                            return Err(IndexerError::HandlerError);
                        }
                    }
                };
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
    events: HashMap<ReceiptEvent, Vec<String>>,
    db: Arc<Mutex<Database>>,
}

impl WasmIndexExecutor {
    pub async fn new(
        db_conn: String,
        manifest: Manifest,
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

        let mut events = HashMap::new();

        for handler in manifest.handlers {
            let handlers = events.entry(handler.event).or_insert_with(Vec::new);

            if !instance.exports.contains(&handler.handler) {
                return Err(IndexerError::MissingHandler(handler.handler));
            }

            handlers.push(handler.handler);
        }

        Ok(WasmIndexExecutor {
            instance,
            _module: module,
            _store: store,
            events,
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
    async fn trigger_event(
        &self,
        event_name: ReceiptEvent,
        bytes: Vec<Vec<u8>>,
        _receipt: Option<Receipt>,
    ) -> IndexerResult<()> {
        let mut args = Vec::with_capacity(bytes.len());
        for arg in bytes.into_iter() {
            args.push(ffi::WasmArg::new(&self.instance, arg)?)
        }
        let arg_list = ffi::WasmArgList::new(&self.instance, args.iter().collect())?;

        if let Some(handlers) = self.events.get(&event_name) {
            for handler in handlers.iter() {
                let fun = self
                    .instance
                    .exports
                    .get_native_function::<(u32, u32, u32), ()>(handler)?;

                self.db.lock().await.start_transaction().await?;

                let ptrs = arg_list.get_ptrs();
                let lens = arg_list.get_lens();
                let len = arg_list.get_len();

                let res = spawn_blocking(
                    move || fun.call(ptrs, lens, len)
                ).await?;

                if let Err(e) = res {
                    error!("Indexer failed {e:?}");
                    self.db.lock().await.revert_transaction().await?;
                    return Err(IndexerError::RuntimeError(e));
                } else {
                    self.db.lock().await.commit_transaction().await?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Connection, Row};
    use fuels_abigen_macro::abigen;
    use fuels_core::{abi_encoder::ABIEncoder, Tokenizable};

    const DATABASE_URL: &str = "postgres://postgres:my-secret@127.0.0.1:5432";
    const MANIFEST: &str = include_str!("test_data/manifest.yaml");
    const BAD_MANIFEST: &str = include_str!("test_data/bad_manifest.yaml");
    const WASM_BYTES: &[u8] = include_bytes!("test_data/simple_wasm.wasm");

    abigen!(MyContract, "indexer/src/test_data/my_struct.json");

    #[derive(Debug)]
    struct Thing1 {
        id: i64,
        account: String,
    }

    #[tokio::test]
    async fn test_wasm_executor() {
        let manifest: Manifest = serde_yaml::from_str(MANIFEST).expect("Bad yaml file.");
        let bad_manifest: Manifest = serde_yaml::from_str(BAD_MANIFEST).expect("Bad yaml file.");

        let executor = WasmIndexExecutor::new(DATABASE_URL.to_string(), bad_manifest, WASM_BYTES).await;
        match executor {
            Err(IndexerError::MissingHandler(o)) if o == "fn_one" => (),
            e => panic!("Expected missing handler error {:#?}", e),
        }

        let executor = WasmIndexExecutor::new(DATABASE_URL.to_string(), manifest, WASM_BYTES).await;
        assert!(executor.is_ok());

        let executor = executor.unwrap();

        let result = executor.trigger_event(
            ReceiptEvent::an_event_name,
            vec![b"ejfiaiddiie".to_vec()],
            None,
        ).await;
        match result {
            Err(IndexerError::RuntimeError(_)) => (),
            e => panic!("Should have been a runtime error {:#?}", e),
        }

        let evt1 = SomeEvent {
            id: 1020,
            account: [0xaf; 32],
        };
        let evt2 = AnotherEvent {
            id: 100,
            hash: [0x43; 32],
            bar: true,
        };

        let encoded = vec![
            ABIEncoder::new()
                .encode(&[evt1.into_token()])
                .expect("Failed to encode"),
            ABIEncoder::new()
                .encode(&[evt2.into_token()])
                .expect("Failed to encode"),
        ];

        let result = executor.trigger_event(ReceiptEvent::an_event_name, encoded, None).await;
        assert!(result.is_ok());

        let mut conn = sqlx::PgConnection::connect(DATABASE_URL).await.expect("Database connection failed!");

        let row = sqlx::query("select id,account from test_namespace.thing1 where id = 1020;").fetch_one(&mut conn).await.expect("Database query failed");

        let id = row.get(0);
        let account = row.get(1);

        let data = Thing1 {
            id,
            account,
        };

        assert_eq!(data.id, 1020);
        assert_eq!(
            data.account,
            "afafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafaf"
        );
    }
}
