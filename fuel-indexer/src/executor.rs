use crate::database::Database;
use crate::ffi;
use crate::{IndexerError, IndexerRequest, IndexerResponse, IndexerResult, Manifest};
use async_std::sync::{Arc, Mutex};
use async_trait::async_trait;
use fuel_indexer_schema::{
    types::fuel::BlockData,
    utils::{deserialize, serialize},
};
use std::path::Path;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::{net::TcpStream, task::spawn_blocking};
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

pub struct NativeIndexExecutor {
    db: Arc<Mutex<Database>>,
    #[allow(dead_code)]
    manifest: Manifest,
    _process: tokio::process::Child,
    stream: TcpStream,
}

impl NativeIndexExecutor {
    pub async fn new(
        db_conn: &str,
        manifest: Manifest,
        path: String,
    ) -> IndexerResult<Self> {
        let db = Arc::new(Mutex::new(
            Database::new(db_conn)
                .await
                .expect("Failed to connect to database"),
        ));

        db.lock().await.load_schema_native(manifest.clone()).await?;

        let mut process = tokio::process::Command::new(path)
            .kill_on_drop(true)
            .spawn()?;

        let mut reader = BufReader::new(
            process
                .stdout
                .take()
                .ok_or(IndexerError::ExecutorInitError)?,
        );
        let mut out = String::new();
        reader.read_line(&mut out).await?;

        let port: u16 = out.parse()?;

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

        let msg = serialize(&IndexerResponse::Blocks(blocks));

        self.stream.write_u64(msg.len() as u64).await?;
        self.stream.write_all(&msg).await?;

        loop {
            let size = self.stream.read_u64().await? as usize;

            if self.stream.read_exact(&mut buf[..size]).await? < size {
                return Err(IndexerError::HandlerError);
            }

            let object: IndexerRequest = deserialize(&buf[..size])
                .expect("Could not deserialize message from indexer.");

            match object {
                IndexerRequest::GetObject(type_id, object_id) => {
                    let object =
                        self.db.lock().await.get_object(type_id, object_id).await;
                    if let Some(obj) = object {
                        self.stream.write_u64(obj.len() as u64).await?;
                        self.stream.write_all(&obj).await?
                    } else {
                        self.stream.write_u64(0).await?;
                    }
                }
                IndexerRequest::PutObject(type_id, bytes, columns) => {
                    self.db
                        .lock()
                        .await
                        .put_object(type_id, columns, bytes)
                        .await;
                }
                IndexerRequest::Commit => break,
            }
        }
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
            error!("WasmIndexExecutor handle_events failed: {e:?}.");
            self.db.lock().await.revert_transaction().await?;
            return Err(IndexerError::RuntimeError(e));
        } else {
            self.db.lock().await.commit_transaction().await?;
        }
        Ok(())
    }
}
