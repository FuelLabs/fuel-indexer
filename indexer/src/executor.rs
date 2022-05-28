use crate::database::Database;
use crate::ffi;
use crate::{IndexerError, IndexerResult, Manifest};
use fuel_indexer_handler::{Handler, HandlerError, Logger};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
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

type ConcurrentHandler = Arc<dyn Handler + Send + Sync>;

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
pub struct IndexExecutor {
    instance: Instance,
    _module: Module,
    _store: Store,
    events: HashMap<String, Vec<String>>,
    db: Arc<Mutex<Database>>,
}

pub struct SimpleIndexExecutor {
    handles: HashMap<String, Vec<ConcurrentHandler>>,
    events: HashMap<String, String>,
    db: Arc<Mutex<Database>>,
}

impl SimpleIndexExecutor {
    pub fn new(db_conn: String, manifest: Manifest) -> Self {
        let events = HashMap::new();
        let db = Arc::new(Mutex::new(Database::new(&db_conn).unwrap()));
        let handles = HashMap::new();

        Self {
            events,
            db,
            handles,
        }
    }

    pub fn register_handler(&mut self, event: String, handler_name: String, h: ConcurrentHandler) {
        if let Some(handler_set_key) = self.events.get(&event) {
            let handles = self.handles.get_mut(handler_set_key).unwrap();
            handles.push(h);
        }
        self.events.insert(event, handler_name);
    }

    pub fn from_file(_index: &Path) -> IndexerResult<IndexExecutor> {
        unimplemented!()
    }

    pub fn trigger_event(&self, event_name: &str, bytes: Vec<Vec<u8>>) -> IndexerResult<()> {
        if let Some(handle_key) = self.events.get(event_name) {
            if let Some(handles) = self.handles.get(handle_key) {
                for handler in handles.iter() {
                    self.db.lock().expect("Lock poisoned").start_transaction()?;
                    let handler = handler.clone();

                    let res = handler.call(bytes.clone());
                    // let res: Result<Vec<u8>, HandlerError> = Ok(Vec::new());

                    if let Err(e) = res {
                        error!("Indexer failed {e:?}");
                        self.db
                            .lock()
                            .expect("Lock poisoned")
                            .revert_transaction()?;
                        return Err(IndexerError::HandlerError(e));
                    } else {
                        self.db
                            .lock()
                            .expect("Lock poisoned")
                            .commit_transaction()?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl IndexExecutor {
    pub fn new(
        db_conn: String,
        manifest: Manifest,
        wasm_bytes: impl AsRef<[u8]>,
    ) -> IndexerResult<IndexExecutor> {
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
            .load_schema(&instance)?;

        let mut events = HashMap::new();

        for handler in manifest.handlers {
            let handles = events.entry(handler.event).or_insert_with(Vec::new);

            if !instance.exports.contains(&handler.handler) {
                return Err(IndexerError::MissingHandler(handler.handler));
            }

            handles.push(handler.handler);
        }

        Ok(IndexExecutor {
            instance,
            _module: module,
            _store: store,
            events,
            db: env.db.clone(),
        })
    }

    /// Restore index from wasm file
    pub fn from_file(_index: &Path) -> IndexerResult<IndexExecutor> {
        unimplemented!()
    }

    /// Trigger a WASM event handler, passing in a serialized event struct.
    pub fn trigger_event(&self, event_name: &str, bytes: Vec<Vec<u8>>) -> IndexerResult<()> {
        let mut args = Vec::with_capacity(bytes.len());
        for arg in bytes.into_iter() {
            args.push(ffi::WasmArg::new(&self.instance, arg)?)
        }
        let arg_list = ffi::WasmArgList::new(&self.instance, args.iter().collect())?;

        if let Some(handles) = self.events.get(event_name) {
            for handler in handles.iter() {
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
    use fuels_core::abi_encoder::ABIEncoder;

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

        let executor = IndexExecutor::new(DATABASE_URL.to_string(), bad_manifest, WASM_BYTES);
        match executor {
            Err(IndexerError::MissingHandler(o)) if o == "fn_one" => (),
            e => panic!("Expected missing handler error {:#?}", e),
        }

        let executor = IndexExecutor::new(DATABASE_URL.to_string(), manifest, WASM_BYTES);
        assert!(executor.is_ok());

        let executor = executor.unwrap();

        let result = executor.trigger_event("an_event_name", vec![b"ejfiaiddiie".to_vec()]);
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

        let result = executor.trigger_event("an_event_name", encoded);
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
