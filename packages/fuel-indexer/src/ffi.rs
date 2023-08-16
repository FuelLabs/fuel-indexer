use async_std::sync::MutexGuard;
use fuel_indexer_lib::defaults;
use fuel_indexer_schema::{join::RawQuery, FtColumn};
use fuel_indexer_types::ffi::{
    LOG_LEVEL_DEBUG, LOG_LEVEL_ERROR, LOG_LEVEL_INFO, LOG_LEVEL_TRACE, LOG_LEVEL_WARN,
};
use thiserror::Error;
use tracing::{debug, error, info, trace, warn};
use wasmer::{
    ExportError, Exports, Function, FunctionEnvMut, Instance, MemoryView, RuntimeError,
    Store, StoreMut, WasmPtr,
};
use wasmer_middlewares::metering::{
    get_remaining_points, set_remaining_points, MeteringPoints,
};

use crate::{IndexEnv, IndexerResult};

/// Symbol name for the module entrypoint.
pub const MODULE_ENTRYPOINT: &str = "handle_events";

/// Result type returned by FFI operations.
type FFIResult<T> = Result<T, FFIError>;

/// Error type returned by FFI operations.
#[derive(Debug, Error)]
pub enum FFIError {
    #[error("Invalid memory access")]
    MemoryBound,
    #[error("Error calling into wasm function {0:?}")]
    Runtime(#[from] RuntimeError),
    #[error("Invalid export {0:?}")]
    Export(#[from] ExportError),
    #[error("Expected result from call {0:?}")]
    None(String),
}

/// Get the version of the indexer schema stored in the WASM instance.
pub fn get_version(store: &mut StoreMut, instance: &Instance) -> FFIResult<String> {
    let exports = &instance.exports;

    let ptr = exports.get_function("get_version_ptr")?.call(store, &[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None("get_version".to_string()))? as u32;

    let len = exports.get_function("get_version_len")?.call(store, &[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None("get_version".to_string()))? as u32;

    let memory = exports.get_memory("memory")?.view(store);
    let version = get_string(&memory, ptr, len)?;

    Ok(version)
}

/// Fetch the string at the given pointer from memory.
fn get_string(mem: &MemoryView, ptr: u32, len: u32) -> FFIResult<String> {
    let result = WasmPtr::<u8>::new(ptr)
        .read_utf8_string(mem, len)
        .or(Err(FFIError::MemoryBound))?;
    Ok(result)
}

/// Fetch the object ID at the given pointer from memory.
fn get_object_id(mem: &MemoryView, ptr: u32, len: u32) -> FFIResult<String> {
    let id = get_string(mem, ptr, len).unwrap();
    let id: String = id.chars().filter(|&c| c != '\0').collect();
    Ok(id)
}

/// Log the string at the given pointer to stdout.
fn log_data(mut env: FunctionEnvMut<IndexEnv>, ptr: u32, len: u32, log_level: u32) {
    let (idx_env, store) = env.data_and_store_mut();
    let mem = idx_env
        .memory
        .as_mut()
        .expect("Memory unitialized.")
        .view(&store);

    let log_string =
        get_string(&mem, ptr, len).expect("Log string could not be fetched.");

    match log_level {
        LOG_LEVEL_ERROR => error!("{log_string}",),
        LOG_LEVEL_WARN => warn!("{log_string}",),
        LOG_LEVEL_INFO => info!("{log_string}",),
        LOG_LEVEL_DEBUG => debug!("{log_string}",),
        LOG_LEVEL_TRACE => trace!("{log_string}",),
        l => panic!("Invalid log level: {l}"),
    }
}

/// Fetch the given type at the given pointer from memory.
///
/// This function is fallible, and will panic if the type cannot be fetched.
fn get_object(
    mut env: FunctionEnvMut<IndexEnv>,
    type_id: i64,
    ptr: u32,
    len_ptr: u32,
) -> u32 {
    let (idx_env, mut store) = env.data_and_store_mut();

    let mem = idx_env
        .memory
        .as_mut()
        .expect("Memory unitialized.")
        .view(&store);

    let offset = 1;
    let len = 64;
    let padding = 6;
    let id = get_object_id(&mem, ptr + offset, len + padding + offset).unwrap();

    let rt = tokio::runtime::Handle::current();
    let bytes =
        rt.block_on(async { idx_env.db.lock().await.get_object(type_id, id).await });

    if let Some(bytes) = bytes {
        let alloc_fn = idx_env.alloc.as_mut().expect("Alloc export is missing.");

        let size = bytes.len() as u32;
        let result = alloc_fn.call(&mut store, size).expect("Alloc failed.");
        let range = result as usize..result as usize + size as usize;

        let mem = idx_env
            .memory
            .as_mut()
            .expect("Memory unitialized.")
            .view(&store);
        WasmPtr::<u32>::new(len_ptr)
            .deref(&mem)
            .write(size)
            .expect("Failed to write length to memory.");

        unsafe {
            mem.data_unchecked_mut()[range].copy_from_slice(&bytes);
        }

        result
    } else {
        0
    }
}

/// Put the given type at the given pointer into memory.
///
/// This function is fallible, and will panic if the type cannot be saved.
fn put_object(mut env: FunctionEnvMut<IndexEnv>, type_id: i64, ptr: u32, len: u32) {
    let (idx_env, store) = env.data_and_store_mut();
    let mem = idx_env
        .memory
        .as_mut()
        .expect("Memory unitialized")
        .view(&store);

    let mut bytes = Vec::with_capacity(len as usize);
    let range = ptr as usize..ptr as usize + len as usize;

    unsafe {
        bytes.extend_from_slice(&mem.data_unchecked()[range]);
    }

    let columns: Vec<FtColumn> = match bincode::deserialize(&bytes) {
        Ok(columns) => columns,
        Err(e) => {
            error!("Failed to deserialize Vec<FtColumn> for put_object: {e:?}",);
            return;
        }
    };

    let rt = tokio::runtime::Handle::current();
    rt.block_on(async {
        idx_env
            .db
            .lock()
            .await
            .put_object(type_id, columns, bytes)
            .await
    });
}

/// Execute the arbitrary query at the given pointer.
///
/// This function is fallible, and will panic if the query cannot be executed.
fn put_many_to_many_record(mut env: FunctionEnvMut<IndexEnv>, ptr: u32, len: u32) {
    let (idx_env, store) = env.data_and_store_mut();
    let mem = idx_env
        .memory
        .as_mut()
        .expect("Memory unitialized")
        .view(&store);

    let mut bytes = Vec::with_capacity(len as usize);
    let range = ptr as usize..ptr as usize + len as usize;

    unsafe {
        bytes.extend_from_slice(&mem.data_unchecked()[range]);
    }

    let queries: Vec<RawQuery> =
        bincode::deserialize(&bytes).expect("Failed to deserialize queries");
    let queries = queries.iter().map(|q| q.to_string()).collect::<Vec<_>>();
    let rt = tokio::runtime::Handle::current();
    rt.block_on(async {
        idx_env
            .db
            .lock()
            .await
            .put_many_to_many_record(queries)
            .await
    });
}

/// Get the exports for the given store and environment.
pub fn get_exports(store: &mut Store, env: &wasmer::FunctionEnv<IndexEnv>) -> Exports {
    let mut exports = Exports::new();

    let f_get_obj = Function::new_typed_with_env(store, env, get_object);
    let f_put_obj = Function::new_typed_with_env(store, env, put_object);
    let f_log_data = Function::new_typed_with_env(store, env, log_data);
    let f_put_many_to_many_record =
        Function::new_typed_with_env(store, env, put_many_to_many_record);

    exports.insert("ff_get_object".to_string(), f_get_obj);
    exports.insert("ff_put_object".to_string(), f_put_obj);
    exports.insert(
        "ff_put_many_to_many_record".to_string(),
        f_put_many_to_many_record,
    );
    exports.insert("ff_log_data".to_string(), f_log_data);

    exports
}

/// Holds on to a byte blob that has been copied into WASM memory until
/// it's not needed anymore, then tells WASM to deallocate.
pub(crate) struct WasmArg<'a> {
    /// WASM store.
    store: MutexGuard<'a, Store>,

    /// The WASM instance.
    instance: Instance,

    /// Pointer to the start of the byte blob in WASM memory.
    ptr: u32,

    /// Length of the byte blob.
    len: u32,

    /// Whether metering is enabled.
    metering_enabled: bool,
}

impl<'a> WasmArg<'a> {
    /// Create a new `WasmArg` from the given bytes.
    #[allow(clippy::result_large_err)]
    pub fn new(
        mut store: MutexGuard<'a, Store>,
        instance: Instance,
        bytes: Vec<u8>,
        metering_enabled: bool,
    ) -> IndexerResult<WasmArg<'a>> {
        let alloc_fn = instance
            .exports
            .get_typed_function::<u32, u32>(&store, "alloc_fn")?;

        let len = bytes.len() as u32;
        let ptr = alloc_fn.call(&mut store, len)?;
        let range = ptr as usize..(ptr + len) as usize;

        let memory = instance.exports.get_memory("memory")?.view(&store);
        unsafe {
            memory.data_unchecked_mut()[range].copy_from_slice(&bytes);
        }

        Ok(WasmArg {
            store,
            instance,
            ptr,
            len,
            metering_enabled,
        })
    }

    pub fn store(&mut self) -> &mut Store {
        &mut self.store
    }

    /// Get the pointer to the start of the byte blob in WASM memory.
    pub fn get_ptr(&self) -> u32 {
        self.ptr
    }

    /// Get the length of the byte blob.
    pub fn get_len(&self) -> u32 {
        self.len
    }
}

impl<'a> Drop for WasmArg<'a> {
    /// Drop the byte blob from WASM memory.
    fn drop(&mut self) {
        let dealloc_fn = self
            .instance
            .exports
            .get_typed_function::<(u32, u32), ()>(&self.store, "dealloc_fn")
            .expect("No dealloc fn");

        // Need to track whether metering is enabled or otherwise getting or setting points will panic
        if self.metering_enabled {
            let pts = match get_remaining_points(&mut self.store, &self.instance) {
                MeteringPoints::Exhausted => 0,
                MeteringPoints::Remaining(pts) => pts,
            };
            set_remaining_points(
                &mut self.store,
                &self.instance,
                defaults::METERING_POINTS,
            );
            dealloc_fn
                .call(&mut self.store, self.ptr, self.len)
                .expect("Dealloc failed");
            set_remaining_points(&mut self.store, &self.instance, pts);
        } else {
            dealloc_fn
                .call(&mut self.store, self.ptr, self.len)
                .expect("Dealloc failed");
        }
    }
}
