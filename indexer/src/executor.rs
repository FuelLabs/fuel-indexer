use crate::database::Database;
use crate::ffi;
use crate::handler::{CustomHandler, ReceiptEvent};
use crate::{IndexerError, IndexerResult, Manifest};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::{error, info};
use wasmer::{
    imports, Instance, LazyInit, Memory, Module, NativeFunc, RuntimeError, Store, WasmerEnv,
};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;
fn compiler() -> Cranelift {
    Cranelift::default()
}

pub trait Executor
where
    Self: Sized,
{
    fn from_file(index: &Path) -> IndexerResult<Self>;
    fn trigger_event(&self, event_name: ReceiptEvent, bytes: Vec<Vec<u8>>) -> IndexerResult<()>;
}

#[derive(Error, Debug)]
pub enum TxError {
    #[error("Diesel Error {0:?}")]
    DieselError(#[from] diesel::result::Error),
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

pub struct CustomIndexExecutor {
    events: HashMap<ReceiptEvent, Vec<CustomHandler>>,
    db: Arc<Mutex<Database>>,
    #[allow(dead_code)]
    manifest: Manifest,
}

impl CustomIndexExecutor {
    pub fn new(db_conn: &str, manifest: Manifest) -> IndexerResult<Self> {
        let events = HashMap::new();
        let db = Arc::new(Mutex::new(Database::new(db_conn).unwrap()));

        db.lock()
            .expect("Lock poisoned from schema load")
            .load_schema_custom(manifest.clone())?;

        Ok(Self {
            events,
            db,
            manifest,
        })
    }

    pub fn handler(&mut self, handler: CustomHandler) -> &mut Self {
        info!("Registering new handler: {:?}", handler);

        let event = handler.event.clone();
        match self.events.get_mut(&event) {
            Some(handles) => {
                handles.push(handler);
            }
            None => {
                let h = vec![handler];
                self.events.insert(event, h);
            }
        }

        self
    }
}

impl Executor for CustomIndexExecutor {
    fn from_file(_index: &Path) -> IndexerResult<Self> {
        unimplemented!()
    }

    fn trigger_event(&self, event: ReceiptEvent, bytes: Vec<Vec<u8>>) -> IndexerResult<()> {
        if let Some(handles) = self.events.get(&event) {
            for handler in handles.iter() {
                self.db
                    .lock()
                    .expect("Lock poisoned on tx start")
                    .start_transaction()?;

                #[allow(clippy::clone_double_ref)]
                let handler = handler.clone();

                let func = handler.handle;

                match func(bytes.clone()) {
                    Ok(res) => {
                        if let Some(res) = res {
                            self.db.lock().expect("Lock poisoned on handle").put_object(
                                handler.type_id,
                                res.columns,
                                res.data,
                                false,
                            );

                            self.db
                                .lock()
                                .expect("Lock poisoned")
                                .commit_transaction()?;
                        }
                    }
                    Err(e) => {
                        error!("Indexer failed {e:?}");
                        self.db
                            .lock()
                            .expect("Lock poisoned on error")
                            .revert_transaction()?;
                        return Err(IndexerError::HandlerError);
                    }
                }
            }
        }
        Ok(())
    }
}

impl IndexEnv {
    pub fn new(db_conn: String) -> IndexerResult<IndexEnv> {
        let db = Arc::new(Mutex::new(Database::new(&db_conn)?));
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
    pub fn new(
        db_conn: String,
        manifest: Manifest,
        wasm_bytes: impl AsRef<[u8]>,
    ) -> IndexerResult<Self> {
        let store = Store::new(&Universal::new(compiler()).engine());
        let module = Module::new(&store, &wasm_bytes)?;

        let mut import_object = imports! {};

        let mut env = IndexEnv::new(db_conn)?;
        let exports = ffi::get_exports(&env, &store);
        import_object.register("env", exports);

        let instance = Instance::new(&module, &import_object)?;
        env.init_with_instance(&instance)?;
        env.db
            .lock()
            .expect("mutex lock failed")
            .load_schema_wasm(&instance)?;

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

impl Executor for WasmIndexExecutor {
    /// Restore index from wasm file
    fn from_file(_index: &Path) -> IndexerResult<Self> {
        unimplemented!()
    }

    /// Trigger a WASM event handler, passing in a serialized event struct.
    fn trigger_event(&self, event_name: ReceiptEvent, bytes: Vec<Vec<u8>>) -> IndexerResult<()> {
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

                self.db.lock().expect("Lock poisoned").start_transaction()?;

                let res = fun.call(arg_list.get_ptrs(), arg_list.get_lens(), arg_list.get_len());

                if let Err(e) = res {
                    error!("Indexer failed {e:?}");
                    self.db
                        .lock()
                        .expect("Lock poisoned")
                        .revert_transaction()?;
                    return Err(IndexerError::RuntimeError(e));
                } else {
                    self.db
                        .lock()
                        .expect("Lock poisoned")
                        .commit_transaction()?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(feature = "postgres")]
#[cfg(test)]
mod tests {
    use super::*;
    use diesel::sql_types::*;
    use diesel::{
        prelude::PgConnection, sql_query, Connection, Queryable, QueryableByName, RunQueryDsl,
    };
    use fuels_abigen_macro::abigen;
    use fuels_core::{abi_encoder::ABIEncoder, Tokenizable};

    const DATABASE_URL: &str = "postgres://postgres:my-secret@127.0.0.1:5432";
    const MANIFEST: &str = include_str!("test_data/manifest.yaml");
    const BAD_MANIFEST: &str = include_str!("test_data/bad_manifest.yaml");
    const WASM_BYTES: &[u8] = include_bytes!("test_data/simple_wasm.wasm");

    abigen!(MyContract, "indexer/src/test_data/my_struct.json");

    #[derive(Debug, Queryable, QueryableByName)]
    struct Thing1 {
        #[sql_type = "BigInt"]
        id: i64,
        #[sql_type = "Text"]
        account: String,
    }

    #[test]
    fn test_executor() {
        let manifest: Manifest = serde_yaml::from_str(MANIFEST).expect("Bad yaml file.");
        let bad_manifest: Manifest = serde_yaml::from_str(BAD_MANIFEST).expect("Bad yaml file.");

        let executor = WasmIndexExecutor::new(DATABASE_URL.to_string(), bad_manifest, WASM_BYTES);
        match executor {
            Err(IndexerError::MissingHandler(o)) if o == "fn_one" => (),
            e => panic!("Expected missing handler error {:#?}", e),
        }

        let executor = WasmIndexExecutor::new(DATABASE_URL.to_string(), manifest, WASM_BYTES);
        assert!(executor.is_ok());

        let executor = executor.unwrap();

        let result = executor.trigger_event(ReceiptEvent::Log, vec![b"ejfiaiddiie".to_vec()]);
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

        let result = executor.trigger_event(ReceiptEvent::Log, encoded);
        assert!(result.is_ok());

        let conn = PgConnection::establish(DATABASE_URL).expect("Postgres connection failed");
        let data: Vec<Thing1> =
            sql_query("select id,account from test_namespace.thing1 where id = 1020;")
                .load(&conn)
                .expect("Database query failed");

        assert_eq!(data.len(), 1);
        assert_eq!(data[0].id, 1020);
        assert_eq!(
            data[0].account,
            "afafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafaf"
        );
    }
}
