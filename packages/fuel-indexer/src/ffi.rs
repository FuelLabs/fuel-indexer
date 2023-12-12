use async_std::sync::MutexGuard;
use fuel_indexer_lib::{defaults, WasmIndexerError};
use fuel_indexer_schema::{join::RawQuery, FtColumn};
use fuel_indexer_types::ffi::{
    LOG_LEVEL_DEBUG, LOG_LEVEL_ERROR, LOG_LEVEL_INFO, LOG_LEVEL_TRACE, LOG_LEVEL_WARN,
};
use thiserror::Error;
use tracing::{debug, error, info, trace, warn};
use wasmer::{
    AsStoreMut, ExportError, Exports, Function, FunctionEnvMut, Instance, MemoryView,
    RuntimeError, Store, WasmPtr,
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

/// Get a string stored in the WASM module.
fn get_string_from_instance(
    store: &mut Store,
    instance: &Instance,
    ptr_fn_name: &str,
    len_fn_name: &str,
) -> FFIResult<String> {
    let exports = &instance.exports;

    let ptr = exports
        .get_function(ptr_fn_name)?
        .call(&mut store.as_store_mut(), &[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None(ptr_fn_name.to_string()))? as u32;

    let len = exports
        .get_function(len_fn_name)?
        .call(&mut store.as_store_mut(), &[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None(len_fn_name.to_string()))? as u32;

    let memory = exports.get_memory("memory")?.view(store);

    let result = get_string(&memory, ptr, len)?;

    Ok(result)
}

/// Get the version of the indexer schema stored in the WASM instance.
pub fn get_error_message(store: &mut Store, instance: &Instance) -> FFIResult<String> {
    get_string_from_instance(
        store,
        instance,
        "get_error_message_ptr",
        "get_error_message_len",
    )
}

/// Get the version of the indexer schema stored in the WASM instance.
pub fn get_version(store: &mut Store, instance: &Instance) -> FFIResult<String> {
    get_string_from_instance(store, instance, "get_version_ptr", "get_version_len")
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
    let id = get_string(mem, ptr, len)?;
    // https://stackoverflow.com/a/1348551
    let id: String = id.chars().filter(|&c| c != '\0').collect();
    Ok(id)
}

/// Log the string at the given pointer to stdout.
fn log_data(
    mut env: FunctionEnvMut<IndexEnv>,
    ptr: u32,
    len: u32,
    log_level: u32,
) -> Result<(), WasmIndexerError> {
    let (idx_env, store) = env.data_and_store_mut();

    if idx_env
        .kill_switch
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        // If the kill switch has been flipped, returning an error will cause an
        // early termination of WASM execution.
        return Err(WasmIndexerError::KillSwitch);
    }

    let mem = idx_env
        .memory
        .as_mut()
        .ok_or(WasmIndexerError::UninitializedMemory)?
        .view(&store);

    let log_string =
        get_string(&mem, ptr, len).expect("Log string could not be fetched.");

    match log_level {
        LOG_LEVEL_ERROR => error!("{log_string}",),
        LOG_LEVEL_WARN => warn!("{log_string}",),
        LOG_LEVEL_INFO => info!("{log_string}",),
        LOG_LEVEL_DEBUG => debug!("{log_string}",),
        LOG_LEVEL_TRACE => trace!("{log_string}",),
        l => {
            error!("Invalid log level: {l}");
            return Err(WasmIndexerError::InvalidLogLevel);
        }
    }

    Ok(())
}

/// Fetch the given type at the given pointer from memory.
fn get_object(
    mut env: FunctionEnvMut<IndexEnv>,
    type_id: i64,
    ptr: u32,
    len_ptr: u32,
) -> Result<u32, WasmIndexerError> {
    let (idx_env, mut store) = env.data_and_store_mut();

    if idx_env
        .kill_switch
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        // If the kill switch has been flipped, returning an error will cause an
        // early termination of WASM execution.
        return Err(WasmIndexerError::KillSwitch);
    }

    let mem = idx_env
        .memory
        .as_mut()
        .ok_or(WasmIndexerError::UninitializedMemory)?
        .view(&store);

    let offset = 1;
    let len = 64;
    let padding = 6;
    let id = get_object_id(&mem, ptr + offset, len + padding + offset)
        .map_err(|_| WasmIndexerError::GetObjectIdFailed)?;

    let rt = tokio::runtime::Handle::current();
    let bytes = rt
        .block_on(async { idx_env.db.lock().await.get_object(type_id, id).await })
        .map_err(|e| {
            error!("Failed to get_object: {e}");
            WasmIndexerError::DatabaseError
        })?;

    if let Some(bytes) = bytes {
        let alloc_fn = idx_env
            .alloc
            .as_mut()
            .ok_or(WasmIndexerError::AllocMissing)?;

        let size = bytes.len() as u32;
        let result = alloc_fn
            .call(&mut store, size)
            .map_err(|_| WasmIndexerError::AllocFailed)?;
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

        Ok(result)
    } else {
        Ok(0)
    }
}

/// Get multiple objects from the database that satisfy the given constraints.
fn find_many(
    mut env: FunctionEnvMut<IndexEnv>,
    type_id: i64,
    ptr: u32,
    len_ptr: u32,
) -> Result<u32, WasmIndexerError> {
    let (idx_env, mut store) = env.data_and_store_mut();

    if idx_env
        .kill_switch
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        // If the kill switch has been flipped, returning an error will cause an
        // early termination of WASM execution.
        return Err(WasmIndexerError::KillSwitch);
    }

    let mem = idx_env
        .memory
        .as_mut()
        .ok_or(WasmIndexerError::UninitializedMemory)?
        .view(&store);

    let len = WasmPtr::<u32>::new(len_ptr)
        .deref(&mem)
        .read()
        .expect("Failed to read length from memory.");

    let constraints = get_object_id(&mem, ptr + 1, len - 1).unwrap();

    let rt = tokio::runtime::Handle::current();
    let bytes = rt
        .block_on(async {
            idx_env
                .db
                .lock()
                .await
                .find_many(type_id, constraints)
                .await
        })
        .unwrap();

    if !bytes.is_empty() {
        let bytes = fuel_indexer_lib::utils::serialize(&bytes);

        let alloc_fn = idx_env
            .alloc
            .as_mut()
            .ok_or(WasmIndexerError::AllocMissing)?;

        let size = bytes.len() as u32;
        let result = alloc_fn
            .call(&mut store, size)
            .map_err(|_| WasmIndexerError::AllocFailed)?;
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

        Ok(result)
    } else {
        Ok(0)
    }
}

/// Delete multiple objects from the database that satisfy the given constraints.
fn delete(
    mut env: FunctionEnvMut<IndexEnv>,
    type_id: i64,
    ptr: u32,
    len_ptr: u32,
) -> Result<u64, WasmIndexerError> {
    let (idx_env, store) = env.data_and_store_mut();

    if idx_env
        .kill_switch
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        // If the kill switch has been flipped, returning an error will cause an
        // early termination of WASM execution.
        return Err(WasmIndexerError::KillSwitch);
    }

    let mem = idx_env
        .memory
        .as_mut()
        .ok_or(WasmIndexerError::UninitializedMemory)?
        .view(&store);

    let len = WasmPtr::<u32>::new(len_ptr)
        .deref(&mem)
        .read()
        .expect("Failed to read length from memory.");

    let constraints = get_object_id(&mem, ptr + 1, len - 1).unwrap();

    let rt = tokio::runtime::Handle::current();
    let count = rt
        .block_on(async { idx_env.db.lock().await.delete(type_id, constraints).await })
        .unwrap();

    Ok(count as u64)
}

/// Put the given type at the given pointer into memory.
fn put_object(
    mut env: FunctionEnvMut<IndexEnv>,
    type_id: i64,
    ptr: u32,
    len: u32,
) -> Result<(), WasmIndexerError> {
    let (idx_env, store) = env.data_and_store_mut();

    if idx_env
        .kill_switch
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        // If the kill switch has been flipped, returning an error will cause an
        // early termination of WASM execution.
        return Err(WasmIndexerError::KillSwitch);
    }

    let mem = idx_env
        .memory
        .as_mut()
        .ok_or(WasmIndexerError::UninitializedMemory)?
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
            return Err(WasmIndexerError::DeserializationError);
        }
    };

    let rt = tokio::runtime::Handle::current();
    let result = rt.block_on(async {
        idx_env
            .db
            .lock()
            .await
            .put_object(type_id, columns, bytes)
            .await
    });

    if let Err(e) = result {
        error!("Failed to put_object: {e}");
        return Err(database_operation_failure(e));
    };

    Ok(())
}

/// Execute the arbitrary query at the given pointer.
fn put_many_to_many_record(
    mut env: FunctionEnvMut<IndexEnv>,
    ptr: u32,
    len: u32,
) -> Result<(), WasmIndexerError> {
    let (idx_env, store) = env.data_and_store_mut();

    if idx_env
        .kill_switch
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        // If the kill switch has been flipped, returning an error will cause an
        // early termination of WASM execution.
        return Err(WasmIndexerError::KillSwitch);
    }

    let mem = if let Some(memory) = idx_env.memory.as_mut() {
        memory.view(&store)
    } else {
        return Err(WasmIndexerError::UninitializedMemory);
    };

    let mut bytes = Vec::with_capacity(len as usize);
    let range = ptr as usize..ptr as usize + len as usize;

    unsafe {
        bytes.extend_from_slice(&mem.data_unchecked()[range]);
    }

    let queries: Vec<String> = match bincode::deserialize::<Vec<RawQuery>>(&bytes) {
        Ok(queries) => queries.iter().map(|q| q.to_string()).collect(),
        Err(e) => {
            error!("Failed to deserialize queries: {e:?}");
            return Err(WasmIndexerError::DeserializationError);
        }
    };

    let rt = tokio::runtime::Handle::current();
    let result = rt.block_on(async {
        idx_env
            .db
            .lock()
            .await
            .put_many_to_many_record(queries)
            .await
    });

    if let Err(e) = result {
        error!("Failed to put_many_to_many_record: {e:?}");
        return Err(database_operation_failure(e));
    }

    Ok(())
}

// Returns a specialized error code when the database trigger, which ensures
// indexers can't miss blocks, raises an exception. Otherwise, returns an error
// code indicating a generic database operation failure.
fn database_operation_failure(e: crate::IndexerError) -> WasmIndexerError {
    match e {
        crate::IndexerError::SqlxError(e) => {
            if let Some(e) = e.as_database_error() {
                if let Some(e) = e.try_downcast_ref::<sqlx::postgres::PgDatabaseError>() {
                    if let Some(source) = e.r#where() {
                        if source.contains("PL/pgSQL function ensure_block_height_consecutive() line 8 at RAISE") {
                            return WasmIndexerError::MissingBlocksError
                        }
                    }
                }
            }
            WasmIndexerError::DatabaseError
        }
        _ => WasmIndexerError::DatabaseError,
    }
}

/// When called from WASM it will terminate the execution and return the error
/// code.
pub fn early_exit(err_code: u32) -> Result<(), WasmIndexerError> {
    Err(WasmIndexerError::from(err_code))
}

/// Get the exports for the given store and environment.
pub fn get_exports(store: &mut Store, env: &wasmer::FunctionEnv<IndexEnv>) -> Exports {
    let mut exports = Exports::new();

    let f_get_obj = Function::new_typed_with_env(store, env, get_object);
    let f_find_many = Function::new_typed_with_env(store, env, find_many);
    let f_delete = Function::new_typed_with_env(store, env, delete);
    let f_put_obj = Function::new_typed_with_env(store, env, put_object);
    let f_log_data = Function::new_typed_with_env(store, env, log_data);
    let f_put_many_to_many_record =
        Function::new_typed_with_env(store, env, put_many_to_many_record);
    let f_early_exit = Function::new_typed(store, early_exit);

    exports.insert("ff_early_exit".to_string(), f_early_exit);
    exports.insert("ff_get_object".to_string(), f_get_obj);
    exports.insert("ff_find_many".to_string(), f_find_many);
    exports.insert("ff_delete".to_string(), f_delete);
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

    /// Get a mutable reference to the WASM store.
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
