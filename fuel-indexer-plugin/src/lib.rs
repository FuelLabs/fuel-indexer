extern crate alloc;
use alloc::vec::Vec;
use fuel_indexer_schema::{
    utils::{deserialize, serialize},
    FtColumn,
};
use fuel_indexer_types::ffi::{
    LOG_LEVEL_DEBUG, LOG_LEVEL_ERROR, LOG_LEVEL_INFO, LOG_LEVEL_TRACE, LOG_LEVEL_WARN,
};

#[cfg(feature = "native-execution")]
use fuel_indexer_database::{
    queries, IndexerConnection, IndexerConnectionPool, IndexerDatabaseError,
};

#[cfg(feature = "native-execution")]
use tracing::{debug, error, info, trace, warn};

#[cfg(feature = "native-execution")]
use tracing_subscriber::filter::EnvFilter;

pub mod types {
    pub use fuel_indexer_schema::FtColumn;
    pub use fuel_indexer_types::{abi as fuel, tx, *};
}

pub mod utils {
    pub use fuel_indexer_lib::utils::sha256_digest;
}

extern "C" {
    // TODO: error codes? or just panic and let the runtime handle it?
    fn ff_get_object(type_id: u64, ptr: *const u8, len: *mut u8) -> *mut u8;
    fn ff_put_object(type_id: u64, ptr: *const u8, len: u32);
    fn ff_log_data(ptr: *const u8, len: u32, log_level: u32);
}

pub trait Logger {
    fn error(log: &str);
    fn warn(log: &str);
    fn info(log: &str);
    fn debug(log: &str);
    fn trace(log: &str);
}

// TODO: more to do here, hook up to 'impl log::Log for WasmLogger'
pub struct WasmLogger;

impl Logger for WasmLogger {
    fn error(log: &str) {
        unsafe { ff_log_data(log.as_ptr(), log.len() as u32, LOG_LEVEL_ERROR) }
    }

    fn warn(log: &str) {
        unsafe { ff_log_data(log.as_ptr(), log.len() as u32, LOG_LEVEL_WARN) }
    }

    fn info(log: &str) {
        unsafe { ff_log_data(log.as_ptr(), log.len() as u32, LOG_LEVEL_INFO) }
    }

    fn debug(log: &str) {
        unsafe { ff_log_data(log.as_ptr(), log.len() as u32, LOG_LEVEL_DEBUG) }
    }

    fn trace(log: &str) {
        unsafe { ff_log_data(log.as_ptr(), log.len() as u32, LOG_LEVEL_TRACE) }
    }
}

#[cfg(feature = "native-execution")]
pub struct NativeLogger;

#[cfg(feature = "native-execution")]
impl Logger for NativeLogger {
    fn error(log: &str) {
        error!(log);
    }

    fn warn(log: &str) {
        warn!(log);
    }

    fn info(log: &str) {
        info!(log);
    }

    fn debug(log: &str) {
        debug!(log);
    }

    fn trace(log: &str) {
        trace!(log);
    }
}

#[cfg(feature = "native-execution")]
impl NativeLogger {
    pub fn init() -> Result<(), Box<dyn std::error::Error>> {
        let filter = match std::env::var_os("RUST_LOG") {
            Some(_) => {
                EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided")
            }
            None => EnvFilter::new("info"),
        };

        tracing_subscriber::fmt::Subscriber::builder()
            .with_writer(std::io::stderr)
            .with_env_filter(filter)
            .init();

        Ok(())
    }
}

pub trait WasmEntity: Sized + PartialEq + Eq + std::fmt::Debug {
    const TYPE_ID: u64;

    fn from_row(vec: Vec<FtColumn>) -> Self;

    fn to_row(&self) -> Vec<FtColumn>;

    fn type_id(&self) -> u64 {
        Self::TYPE_ID
    }

    // TODO: should take accept any id type, not just u64
    fn load(id: u64) -> Option<Self> {
        unsafe {
            let buf = id.to_le_bytes();
            let mut buflen = (buf.len() as u32).to_le_bytes();

            let ptr = ff_get_object(Self::TYPE_ID, buf.as_ptr(), buflen.as_mut_ptr());

            if !ptr.is_null() {
                let len = u32::from_le_bytes(buflen) as usize;
                let bytes = Vec::from_raw_parts(ptr, len, len);
                let vec = deserialize(&bytes).expect("Bad serialization.");

                Some(Self::from_row(vec))
            } else {
                None
            }
        }
    }

    fn save(&self) {
        unsafe {
            let buf = serialize(&self.to_row());
            ff_put_object(Self::TYPE_ID, buf.as_ptr(), buf.len() as u32)
        }
    }
}

#[cfg(feature = "native-execution")]
pub trait NativeEntity: Sized + PartialEq + Eq + std::fmt::Debug {
    const TYPE_ID: u64;

    fn from_row(vec: Vec<FtColumn>) -> Self;

    fn to_row(&self) -> Vec<FtColumn>;

    fn type_id(&self) -> u64 {
        Self::TYPE_ID
    }

    // TODO: should take accept any id type, not just u64?
    #[allow(unused)]
    fn load(id: u64) -> Option<Self> {
        None
    }

    fn save(&self) {}
}

#[no_mangle]
fn alloc_fn(size: u32) -> *const u8 {
    let vec = Vec::with_capacity(size as usize);
    let ptr = vec.as_ptr();

    core::mem::forget(vec);

    ptr
}

#[no_mangle]
fn dealloc_fn(ptr: *mut u8, len: usize) {
    let _vec = unsafe { Vec::from_raw_parts(ptr, len, len) };
}
