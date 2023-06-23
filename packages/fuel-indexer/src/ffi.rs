use fuel_indexer_schema::FtColumn;
use fuel_indexer_types::ffi::{
    LOG_LEVEL_DEBUG, LOG_LEVEL_ERROR, LOG_LEVEL_INFO, LOG_LEVEL_TRACE, LOG_LEVEL_WARN,
};
use thiserror::Error;
use tracing::{debug, error, info, trace, warn};
use wasmer::{
    ExportError, Exports, Function, FunctionEnv, FunctionEnvMut, Instance, MemoryView,
    RuntimeError, Store, WasmPtr,
};

use crate::{IndexEnv, IndexerResult};
pub const MODULE_ENTRYPOINT: &str = "handle_events";

#[derive(Debug, Error)]
pub enum FFIError {
    #[error("Invalid memory access")]
    MemoryBound,
    #[error("Error calling into wasm function {0:?}")]
    Runtime(#[from] RuntimeError),
    // #[error("Error initializing host environment {0:?}")]
    // HostEnvInit(#[from] HostEnvInitError),
    #[error("Invalid export {0:?}")]
    Export(#[from] ExportError),
    #[error("Expected result from call {0:?}")]
    None(String),
}

pub(crate) fn get_namespace(
    store: &mut Store,
    instance: &Instance,
) -> Result<String, FFIError> {
    let exports = &instance.exports;
    let memory = exports.get_memory("memory")?.view(store);

    let ptr = exports
        .get_function("get_namespace_ptr")?
        .call(store, &[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None("get_namespace".to_string()))? as u32;

    let len = exports
        .get_function("get_namespace_len")?
        .call(store, &[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None("get_namespace".to_string()))? as u32;

    let namespace = get_string(&memory, ptr, len)?;

    Ok(namespace)
}

pub(crate) fn get_identifier(
    store: &mut Store,
    instance: &Instance,
) -> Result<String, FFIError> {
    let exports = &instance.exports;
    let memory = exports.get_memory("memory")?.view(store);

    let ptr = exports
        .get_function("get_identifier_ptr")?
        .call(store, &[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None("get_identifier".to_string()))?
        as u32;

    let len = exports
        .get_function("get_identifier_len")?
        .call(store, &[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None("get_identifier".to_string()))?
        as u32;

    let identifier = get_string(&memory, ptr, len)?;

    Ok(identifier)
}

pub(crate) fn get_version(
    store: &mut Store,
    instance: &Instance,
) -> Result<String, FFIError> {
    let exports = &instance.exports;
    let memory = exports.get_memory("memory")?.view(store);

    let ptr = exports.get_function("get_version_ptr")?.call(store, &[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None("get_version".to_string()))? as u32;

    let len = exports.get_function("get_version_len")?.call(store, &[])?[0]
        .i32()
        .ok_or_else(|| FFIError::None("get_version".to_string()))? as u32;

    let version = get_string(&memory, ptr, len)?;

    Ok(version)
}

fn get_string(mem: &MemoryView, ptr: u32, len: u32) -> Result<String, FFIError> {
    let result = WasmPtr::<u8, wasmer::Array>::new(ptr)
        .get_utf8_string(mem, len)
        .ok_or(FFIError::MemoryBound)?;
    Ok(result)
}

fn get_object_id(mem: &MemoryView, ptr: u32) -> u64 {
    // TODO: get rid of unwrap
    WasmPtr::<u64>::new(ptr).deref(mem).read().unwrap()
    // .expect("Failed to  WasmPtrs.")
    // .get()
}

fn log_data(env: FunctionEnvMut<IndexEnv>, ptr: u32, len: u32, log_level: u32) {
    let mem = env.data().memory.expect("Memory uninitialized.");
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

fn get_object(
    env: FunctionEnvMut<IndexEnv>,
    type_id: i64,
    ptr: u32,
    len_ptr: u32,
) -> u32 {
    let mem = env.data().memory.expect("Memory uninitialized").view();
    // let mem = env.data().memory_ref().expect("Memory uninitialized.");

    let id = get_object_id(mem, ptr);

    // TODO: stash this thing somewhere??
    let rt = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");
    let bytes =
        rt.block_on(async { env.data().db.lock().await.get_object(type_id, id).await });

    if let Some(bytes) = bytes {
        let alloc_fn = env.data().alloc.expect("Alloc export is missing.");

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

fn put_object(env: FunctionEnvMut<IndexEnv>, type_id: i64, ptr: u32, len: u32) {
    let mem = env
        .data_mut()
        .memory
        .expect("Memory uninitialized.")
        .view(store);

    let mut bytes = Vec::with_capacity(len as usize);
    let range = ptr as usize..ptr as usize + len as usize;

    // bytes.extend_from_slice(&mem)
    unsafe {
        bytes.extend_from_slice(&mem.data_unchecked()[range]);
    }

    let columns: Vec<FtColumn> = bincode::deserialize(&bytes).expect("Serde error.");

    // TODO: stash this??
    let rt = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");
    rt.block_on(async {
        env.data_mut()
            .db
            .lock()
            .await
            .put_object(type_id, columns, bytes)
            .await
    });
}

pub fn get_exports(env: &FunctionEnv<IndexEnv>, store: &mut Store) -> Exports {
    let mut exports = Exports::new();

    let f_get_obj = Function::new_typed_with_env(store, env, get_object);
    let f_put_obj = Function::new_typed_with_env(store, env, put_object);
    let f_log_data = Function::new_typed_with_env(store, env, log_data);
    exports.insert(format!("ff_get_object"), f_get_obj);
    exports.insert(format!("ff_get_object"), f_put_obj);
    exports.insert(format!("ff_get_object"), f_log_data);

    exports
}

/// Holds on to a byte blob that has been copied into WASM memory until
/// it's not needed anymore, then tells WASM to deallocate.
pub(crate) struct WasmArg<'a> {
    instance: &'a Instance,
    store: &'a mut Store,
    ptr: u32,
    len: u32,
}

impl<'a> WasmArg<'a> {
    #[allow(clippy::result_large_err)]
    pub fn new(
        store: &mut Store,
        instance: &Instance,
        bytes: Vec<u8>,
    ) -> IndexerResult<WasmArg<'a>> {
        let alloc_fn = instance
            .exports
            .get_typed_function::<u32, u32>(store, "alloc_fn")?;
        let memory = instance.exports.get_memory("memory")?.view(store);

        let len = bytes.len() as u32;
        let ptr = alloc_fn.call(store, len)?;
        let range = ptr as usize..(ptr + len) as usize;

        memory.write(ptr.into(), &bytes);

        Ok(WasmArg {
            instance,
            store,
            ptr,
            len,
        })
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
            .get_typed_function::<(u32, u32), ()>(&mut self.store, "dealloc_fn")
            .expect("No dealloc fn");
        dealloc_fn
            .call(&mut self.store, self.ptr, self.len)
            .expect("Dealloc failed");
    }
}
