use fuel_indexer_schema::FtColumn;
use fuel_indexer_types::ffi::{
    LOG_LEVEL_DEBUG, LOG_LEVEL_ERROR, LOG_LEVEL_INFO, LOG_LEVEL_TRACE, LOG_LEVEL_WARN,
};
use thiserror::Error;
use tracing::{debug, error, info, trace, warn};
use wasmer::{
    ExportError, Exports, Function, HostEnvInitError, Instance, Memory, RuntimeError,
    Store, WasmPtr,
};

use crate::{IndexEnv, IndexerResult};
pub const MODULE_ENTRYPOINT: &str = "handle_events";

#[derive(Debug, Error)]
pub enum FFIError {
    #[error("Invalid memory access")]
    MemoryBound,
    #[error("Error calling into wasm function {0:?}")]
    Runtime(#[from] RuntimeError),
    #[error("Error initializing host environment {0:?}")]
    HostEnvInit(#[from] HostEnvInitError),
    #[error("Invalid export {0:?}")]
    Export(#[from] ExportError),
    #[error("Expected result from call {0:?}")]
    None(String),
}

macro_rules! declare_export {
    ($name:ident, $ffi_env:ident, $store:ident, $env:ident) => {
        let f = Function::new_native_with_env($store, $env.clone(), $name);
        $ffi_env.insert(format!("ff_{}", stringify!($name)), f);
    };
}

pub(crate) fn get_namespace(instance: &Instance) -> Result<String, FFIError> {
    let exports = &instance.exports;
    let memory = exports.get_memory("memory")?;

    let ptr = exports.get_function("get_namespace_ptr")?.call(&[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None("get_namespace".to_string()))? as u32;

    let len = exports.get_function("get_namespace_len")?.call(&[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None("get_namespace".to_string()))? as u32;

    let namespace = get_string(memory, ptr, len)?;

    Ok(namespace)
}

pub(crate) fn get_identifier(instance: &Instance) -> Result<String, FFIError> {
    let exports = &instance.exports;
    let memory = exports.get_memory("memory")?;

    let ptr = exports.get_function("get_identifier_ptr")?.call(&[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None("get_identifier".to_string()))?
        as u32;

    let len = exports.get_function("get_identifier_len")?.call(&[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None("get_identifier".to_string()))?
        as u32;

    let identifier = get_string(memory, ptr, len)?;

    Ok(identifier)
}

pub(crate) fn get_version(instance: &Instance) -> Result<String, FFIError> {
    let exports = &instance.exports;
    let memory = exports.get_memory("memory")?;

    let ptr = exports.get_function("get_version_ptr")?.call(&[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None("get_version".to_string()))? as u32;

    let len = exports.get_function("get_version_len")?.call(&[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None("get_version".to_string()))? as u32;

    let version = get_string(memory, ptr, len)?;

    Ok(version)
}

fn get_string(mem: &Memory, ptr: u32, len: u32) -> Result<String, FFIError> {
    let result = WasmPtr::<u8, wasmer::Array>::new(ptr)
        .get_utf8_string(mem, len)
        .ok_or(FFIError::MemoryBound)?;
    Ok(result)
}

fn get_object_id(mem: &Memory, ptr: u32) -> u64 {
    WasmPtr::<u64>::new(ptr)
        .deref(mem)
        .expect("Failed to deref WasmPtrs.")
        .get()
}

fn log_data(env: &IndexEnv, ptr: u32, len: u32, log_level: u32) {
    let mem = env.memory_ref().expect("Memory uninitialized.");
    let log_string = get_string(mem, ptr, len).expect("Log string could not be fetched.");

    match log_level {
        LOG_LEVEL_ERROR => error!("{log_string}",),
        LOG_LEVEL_WARN => warn!("{log_string}",),
        LOG_LEVEL_INFO => info!("{log_string}",),
        LOG_LEVEL_DEBUG => debug!("{log_string}",),
        LOG_LEVEL_TRACE => trace!("{log_string}",),
        l => panic!("Invalid log level: {l}"),
    }
}

fn get_object(env: &IndexEnv, type_id: i64, ptr: u32, len_ptr: u32) -> u32 {
    let mem = env.memory_ref().expect("Memory uninitialized.");

    let id = get_object_id(mem, ptr);

    let rt = tokio::runtime::Handle::current();
    let bytes = rt.block_on(async { env.db.lock().await.get_object(type_id, id).await });

    if let Some(bytes) = bytes {
        let alloc_fn = env.alloc_ref().expect("Alloc export is missing.");

        let size = bytes.len() as u32;
        let result = alloc_fn.call(size).expect("Alloc failed.");
        let range = result as usize..result as usize + size as usize;

        WasmPtr::<u32>::new(len_ptr)
            .deref(mem)
            .expect("Failed to deref WasmPtr.")
            .set(size);

        unsafe {
            mem.data_unchecked_mut()[range].copy_from_slice(&bytes);
        }

        result
    } else {
        0
    }
}

fn put_object(env: &IndexEnv, type_id: i64, ptr: u32, len: u32) {
    let mem = env.memory_ref().expect("Memory uninitialized.");

    let mut bytes = Vec::with_capacity(len as usize);
    let range = ptr as usize..ptr as usize + len as usize;

    unsafe {
        bytes.extend_from_slice(&mem.data_unchecked()[range]);
    }

    let columns: Vec<FtColumn> = bincode::deserialize(&bytes).expect("Serde error.");

    let rt = tokio::runtime::Handle::current();
    rt.block_on(async {
        env.db
            .lock()
            .await
            .put_object(type_id, columns, bytes)
            .await
    });
}

pub fn get_exports(env: &IndexEnv, store: &Store) -> Exports {
    let mut exports = Exports::new();
    declare_export!(get_object, exports, store, env);
    declare_export!(put_object, exports, store, env);
    declare_export!(log_data, exports, store, env);
    exports
}

/// Holds on to a byte blob that has been copied into WASM memory until
/// it's not needed anymore, then tells WASM to deallocate.
pub(crate) struct WasmArg<'a> {
    instance: &'a Instance,
    ptr: u32,
    len: u32,
}

impl<'a> WasmArg<'a> {
    #[allow(clippy::result_large_err)]
    pub fn new(instance: &Instance, bytes: Vec<u8>) -> IndexerResult<WasmArg> {
        let alloc_fn = instance
            .exports
            .get_native_function::<u32, u32>("alloc_fn")?;
        let memory = instance.exports.get_memory("memory")?;

        let len = bytes.len() as u32;
        let ptr = alloc_fn.call(len)?;
        let range = ptr as usize..(ptr + len) as usize;

        unsafe {
            memory.data_unchecked_mut()[range].copy_from_slice(&bytes);
        }

        Ok(WasmArg { instance, ptr, len })
    }

    pub fn get_ptr(&self) -> u32 {
        self.ptr
    }

    pub fn get_len(&self) -> u32 {
        self.len
    }
}

impl<'a> Drop for WasmArg<'a> {
    fn drop(&mut self) {
        let dealloc_fn = self
            .instance
            .exports
            .get_native_function::<(u32, u32), ()>("dealloc_fn")
            .expect("No dealloc fn");
        dealloc_fn.call(self.ptr, self.len).expect("Dealloc failed");
    }
}
