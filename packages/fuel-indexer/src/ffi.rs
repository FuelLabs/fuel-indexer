use fuel_indexer_schema::FtColumn;
use fuel_indexer_types::ffi::{
    LOG_LEVEL_DEBUG, LOG_LEVEL_ERROR, LOG_LEVEL_INFO, LOG_LEVEL_TRACE, LOG_LEVEL_WARN,
};
use thiserror::Error;
use tracing::{debug, error, info, trace, warn};
use wasmer::{
    ExportError, Exports, Function, FunctionEnvMut, Instance, MemoryView, RuntimeError,
    Store, WasmPtr,
};
use wasmer_middlewares::metering::{
    get_remaining_points, set_remaining_points, MeteringPoints,
};

use crate::{IndexEnv, IndexerResult};
pub const MODULE_ENTRYPOINT: &str = "handle_events";

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

fn get_string(mem: &MemoryView, ptr: u32, len: u32) -> Result<String, FFIError> {
    let result = WasmPtr::<u8>::new(ptr)
        .read_utf8_string(mem, len)
        .or(Err(FFIError::MemoryBound))?;
    Ok(result)
}

fn get_object_id(mem: &MemoryView, ptr: u32) -> u64 {
    WasmPtr::<u64>::new(ptr)
        .deref(mem)
        .read()
        .expect("Could not read object ID")
}

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

fn get_object(
    mut env: FunctionEnvMut<IndexEnv>,
    type_id: i64,
    ptr: u32,
    len_ptr: u32,
) -> u32 {
    let (idx_env, mut store) = env.data_and_store_mut();

    let id = {
        let mem = idx_env
            .memory
            .as_mut()
            .expect("Memory unitialized.")
            .view(&store);
        get_object_id(&mem, ptr)
    };

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
            .unwrap();

        unsafe {
            mem.data_unchecked_mut()[range].copy_from_slice(&bytes);
        }

        result
    } else {
        0
    }
}

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
            error!(
                "Failed to deserialize Vec<FtColumn> for put_object: {:?}",
                e
            );
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

pub fn get_exports(store: &mut Store, env: &wasmer::FunctionEnv<IndexEnv>) -> Exports {
    let mut exports = Exports::new();

    let f_get_obj = Function::new_typed_with_env(store, env, get_object);
    let f_put_obj = Function::new_typed_with_env(store, env, put_object);
    let f_log_data = Function::new_typed_with_env(store, env, log_data);
    exports.insert("ff_get_object".to_string(), f_get_obj);
    exports.insert("ff_put_object".to_string(), f_put_obj);
    exports.insert("ff_log_data".to_string(), f_log_data);

    exports
}

/// Holds on to a byte blob that has been copied into WASM memory until
/// it's not needed anymore, then tells WASM to deallocate.
pub(crate) struct WasmArg<'a> {
    instance: &'a Instance,
    ptr: u32,
    len: u32,
    metering_enabled: bool,
}

impl<'a> WasmArg<'a> {
    #[allow(clippy::result_large_err)]
    pub fn new(
        store: &mut Store,
        instance: &'a Instance,
        bytes: Vec<u8>,
        metering_enabled: bool,
    ) -> IndexerResult<WasmArg<'a>> {
        let alloc_fn = instance
            .exports
            .get_typed_function::<u32, u32>(store, "alloc_fn")?;

        let len = bytes.len() as u32;
        let ptr = alloc_fn.call(store, len)?;
        let range = ptr as usize..(ptr + len) as usize;

        let memory = instance.exports.get_memory("memory")?.view(store);
        unsafe {
            memory.data_unchecked_mut()[range].copy_from_slice(&bytes);
        }

        Ok(WasmArg {
            instance,
            ptr,
            len,
            metering_enabled,
        })
    }

    pub fn get_ptr(&self) -> u32 {
        self.ptr
    }

    pub fn get_len(&self) -> u32 {
        self.len
    }

    pub fn drop(&mut self, store: &mut Store) {
        let dealloc_fn = self
            .instance
            .exports
            .get_typed_function::<(u32, u32), ()>(store, "dealloc_fn")
            .expect("No dealloc fn");
        // Need to track whether metering is enabled or otherwise getting or setting points will panic
        if self.metering_enabled {
            let pts = match get_remaining_points(store, self.instance) {
                MeteringPoints::Exhausted => 0,
                MeteringPoints::Remaining(pts) => pts,
            };
            set_remaining_points(store, self.instance, 1_000_000);
            dealloc_fn
                .call(store, self.ptr, self.len)
                .expect("Dealloc failed");
            set_remaining_points(store, self.instance, pts);
        } else {
            dealloc_fn
                .call(store, self.ptr, self.len)
                .expect("Dealloc failed");
        }
    }
}
