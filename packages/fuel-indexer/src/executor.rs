use crate::database::Database;
use crate::ffi;
use crate::{IndexerError, IndexerResult, Manifest};
use async_std::sync::{Arc, Mutex};
use async_trait::async_trait;
use fuel_indexer_schema::utils::serialize;
use fuel_indexer_types::abi::BlockData;
use futures::Future;
use std::path::Path;
use thiserror::Error;
use tokio::task::spawn_blocking;
use tracing::error;
use wasmer::{
    imports, Instance, LazyInit, Memory, Module, NativeFunc, RuntimeError, Store,
    WasmerEnv,
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

unsafe impl<F: Future + Send> Sync for NativeIndexExecutor<F> {}
unsafe impl<F: Future + Send> Send for NativeIndexExecutor<F> {}

#[allow(dead_code)]
pub struct NativeIndexExecutor<F>
where
    F: Future + Send,
{
    db: Arc<Mutex<Database>>,
    manifest: Manifest,
    handle_events_fn: fn(Vec<BlockData>) -> F,
}

impl<F> NativeIndexExecutor<F>
where
    F: Future + Send,
{
    pub async fn new(
        db_conn: &str,
        manifest: Manifest,
        handle_events_fn: fn(Vec<BlockData>) -> F,
    ) -> IndexerResult<Self> {
        let db = Arc::new(Mutex::new(Database::new(db_conn).await?));
        Ok(Self {
            db,
            manifest,
            handle_events_fn,
        })
    }
}

#[async_trait]
impl<F> Executor for NativeIndexExecutor<F>
where
    F: Future + Send,
{
    async fn handle_events(&mut self, blocks: Vec<BlockData>) -> IndexerResult<()> {
        (self.handle_events_fn)(blocks).await;
        Ok(())
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
        env.db
            .lock()
            .await
            .load_schema(&manifest, Some(&instance))
            .await?;

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

    /// Restore index from wasm file
    pub async fn from_file(db_conn: String, manifest_path: &Path) -> IndexerResult<Self> {
        let manifest = Manifest::from_file(manifest_path)?;
        let bytes = manifest.module_bytes()?;
        Self::new(db_conn, manifest, bytes).await
    }
}

#[async_trait]
impl Executor for WasmIndexExecutor {
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
            error!("WasmIndexExecutor handle_events failed: {}.", e.message());
            let frames = e.trace();
            for (i, frame) in frames.iter().enumerate() {
                println!(
                    "Frame #{}: {:?}::{:?}",
                    i,
                    frame.module_name(),
                    frame.function_name()
                );
            }

            self.db.lock().await.revert_transaction().await?;
            return Err(IndexerError::RuntimeError(e));
        } else {
            self.db.lock().await.commit_transaction().await?;
        }
        Ok(())
    }
}
