extern crate alloc;

use crate::JoinMetadata;
use alloc::vec::Vec;
use fuel_indexer_lib::{
    graphql::MAX_FOREIGN_KEY_LIST_FIELDS,
    join_table_typedefs_name,
    utils::{deserialize, serialize},
};
use fuel_indexer_schema::FtColumn;
use fuel_indexer_types::ffi::*;
use quote::quote;

pub use bincode;
pub use hex::FromHex;
pub use sha2::{Digest, Sha256};
pub use std::collections::{HashMap, HashSet};

extern "C" {
    // TODO: error codes? or just panic and let the runtime handle it?
    fn ff_get_object(type_id: i64, ptr: *const u8, len: *mut u8) -> *mut u8;
    fn ff_put_object(type_id: i64, ptr: *const u8, len: u32);
    fn ff_put_many_to_many_record(ptr: *const u8, len: u32);
    fn ff_log_data(ptr: *const u8, len: u32, log_level: u32);
}

// TODO: more to do here, hook up to 'impl log::Log for Logger'
pub struct Logger;

impl Logger {
    pub fn error(log: &str) {
        unsafe { ff_log_data(log.as_ptr(), log.len() as u32, LOG_LEVEL_ERROR) }
    }

    pub fn warn(log: &str) {
        unsafe { ff_log_data(log.as_ptr(), log.len() as u32, LOG_LEVEL_WARN) }
    }

    pub fn info(log: &str) {
        unsafe { ff_log_data(log.as_ptr(), log.len() as u32, LOG_LEVEL_INFO) }
    }

    pub fn debug(log: &str) {
        unsafe { ff_log_data(log.as_ptr(), log.len() as u32, LOG_LEVEL_DEBUG) }
    }

    pub fn trace(log: &str) {
        unsafe { ff_log_data(log.as_ptr(), log.len() as u32, LOG_LEVEL_TRACE) }
    }
}

pub trait Entity<'a>: Sized + PartialEq + Eq + std::fmt::Debug {
    const TYPE_ID: i64;
    const JOIN_METADATA: Option<[Option<JoinMetadata<'a>>; MAX_FOREIGN_KEY_LIST_FIELDS]>;

    fn from_row(vec: Vec<FtColumn>) -> Self;

    fn to_row(&self) -> Vec<FtColumn>;

    fn type_id(&self) -> i64 {
        Self::TYPE_ID
    }

    fn save_m2m(&self) {
        if let Some(meta) = Self::JOIN_METADATA {
            let items = meta
                .iter()
                .filter(|x| x.is_some())
                .map(|x| x.clone().unwrap())
                .collect::<Vec<_>>();

            // Use the first item to construct the first part of the query
            let first = &items[0];
            let (parent_typedef_name, child_typedef_name) =
                join_table_typedefs_name(&first.table_name);

            // Since join table columns are _required_ to have an `ID` field,
            let mut query = format!(
                "INSERT INTO {} ({}_{}, {}_{}) VALUES",
                first.table_name,
                first.parent_typedef_name(),
                first.parent_column_type(),
                first.child_typedef_name(),
                first.child_column_type()
            );

            items.iter().for_each(|item| {
                // NOTE: Since join table's require an `ID` field on both the parent, and the child, we
                // can assume this `Entity` has an `id` field.
                let id = match self.to_row()[0] {
                    FtColumn::ID(id) => match id {
                        Some(id) => id,
                        _ => panic!("No ID field found on Entity."),
                    },
                    _ => panic!("No ID field found on Entity."),
                };

                let list_type_field = &self.to_row()[item.child_position];

                query.push_str(format!(" ({}, {}),", id, "").as_str());
            });
            unsafe { ff_put_many_to_many_record(query.as_ptr(), query.len() as u32) }
        }
    }

    fn load(id: u64) -> Option<Self> {
        unsafe {
            let buf = id.to_le_bytes();
            let mut buflen = (buf.len() as u32).to_le_bytes();

            let ptr = ff_get_object(Self::TYPE_ID, buf.as_ptr(), buflen.as_mut_ptr());

            if !ptr.is_null() {
                let len = u32::from_le_bytes(buflen) as usize;
                let bytes = Vec::from_raw_parts(ptr, len, len);
                let vec = deserialize(&bytes).expect("Bad serialization.");

                return Some(Self::from_row(vec));
            }

            None
        }
    }

    fn save(&self) {
        self.save_m2m();
        unsafe {
            let buf = serialize(&self.to_row());
            ff_put_object(Self::TYPE_ID, buf.as_ptr(), buf.len() as u32)
        }
    }
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
