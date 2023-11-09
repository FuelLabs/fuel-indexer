extern crate alloc;

use alloc::vec::Vec;
use fuel_indexer_lib::{
    graphql::MAX_FOREIGN_KEY_LIST_FIELDS,
    utils::{deserialize, serialize},
    WasmIndexerError,
};
use fuel_indexer_schema::{
    join::{JoinMetadata, RawQuery},
    FtColumn,
};
use fuel_indexer_types::{
    ffi::*,
    scalar::{Boolean, Bytes, B256, UID},
};
use sqlparser::ast as sql;

pub use bincode;
pub use hex::FromHex;
pub use sha2::{Digest, Sha256};
pub use std::collections::{HashMap, HashSet};

// These are instantiated with functions which return
// `Result<T, WasmIndexerError>`. `wasmer` unwraps the `Result` and uses the
// `Err` variant for ealy exit.
extern "C" {
    fn ff_get_object(type_id: i64, ptr: *const u8, len: *mut u8) -> *mut u8;
    fn ff_single_select(type_id: i64, ptr: *const u8, len: *mut u8) -> *mut u8;
    fn ff_log_data(ptr: *const u8, len: u32, log_level: u32);
    fn ff_put_object(type_id: i64, ptr: *const u8, len: u32);
    fn ff_put_many_to_many_record(ptr: *const u8, len: u32);
    fn ff_early_exit(err_code: u32);
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

/// Trait for a type entity.
///
/// Any entity type that will be processed through a WASM indexer is required to implement this trait.
pub trait Entity<'a>: Sized + PartialEq + Eq + std::fmt::Debug {
    /// Unique identifier for a type.
    const TYPE_ID: i64;

    /// Necessary metadata for saving an entity's list type fields.
    const JOIN_METADATA: Option<[Option<JoinMetadata<'a>>; MAX_FOREIGN_KEY_LIST_FIELDS]>;

    /// Convert database row representation into an instance of an entity.
    fn from_row(vec: Vec<FtColumn>) -> Self;

    /// Convert an instance of an entity into row representation for use in a database.
    fn to_row(&self) -> Vec<FtColumn>;

    /// Returns an entity's internal type ID.
    fn type_id(&self) -> i64 {
        Self::TYPE_ID
    }

    /// Saves a record that contains a list of multiple elements.
    fn save_many_to_many(&self) {
        if let Some(meta) = Self::JOIN_METADATA {
            let items = meta.iter().filter_map(|x| x.clone()).collect::<Vec<_>>();
            let row = self.to_row();
            let queries = items
                .iter()
                .map(|item| RawQuery::from_metadata(item, &row))
                .filter(|query| !query.is_empty())
                .collect::<Vec<_>>();
            let bytes = serialize(&queries);
            unsafe {
                ff_put_many_to_many_record(bytes.as_ptr(), bytes.len() as u32);
            }
        }
    }

    /// Loads a record given a UID.
    fn load(id: UID) -> Option<Self> {
        Self::load_unsafe(id)
    }

    /// Loads a record through the FFI with the WASM runtime and checks for errors.
    fn load_unsafe(id: UID) -> Option<Self> {
        unsafe {
            let buff = if let Ok(bytes) = bincode::serialize(&id.to_string()) {
                bytes
            } else {
                early_exit(WasmIndexerError::SerializationError);
            };

            let mut bufflen = (buff.len() as u32).to_le_bytes();

            let ptr = ff_get_object(Self::TYPE_ID, buff.as_ptr(), bufflen.as_mut_ptr());

            if !ptr.is_null() {
                let len = u32::from_le_bytes(bufflen) as usize;
                let bytes = Vec::from_raw_parts(ptr, len, len);
                match deserialize(&bytes) {
                    Ok(vec) => Some(Self::from_row(vec)),
                    Err(_) => {
                        early_exit(WasmIndexerError::DeserializationError);
                    }
                };
            }

            None
        }
    }

    /// Finds the first entity that satisfies the given constraints.
    fn find(query: impl Into<Query<Self>>) -> Option<Self> {
        let query: Query<Self> = query.into();
        unsafe {
            let buff = bincode::serialize(&query.to_string()).unwrap();
            let mut bufflen = (buff.len() as u32).to_le_bytes();

            let ptr =
                ff_single_select(Self::TYPE_ID, buff.as_ptr(), bufflen.as_mut_ptr());

            if !ptr.is_null() {
                let len = u32::from_le_bytes(bufflen) as usize;
                let bytes = Vec::from_raw_parts(ptr, len, len);
                let data = deserialize(&bytes).unwrap();
                Some(Self::from_row(data))
            } else {
                None
            }
        }
    }

    /// Saves a record.
    fn save(&self) {
        self.save_unsafe()
    }

    /// Saves a record through the FFI with the WASM runtime and checks for errors.
    fn save_unsafe(&self) {
        unsafe {
            let buf = serialize(&self.to_row());
            ff_put_object(Self::TYPE_ID, buf.as_ptr(), buf.len() as u32);
        }

        self.save_many_to_many()
    }
}

pub struct Query<T> {
    constraint: Constraint<T>,
    order_by: Option<sql::OrderByExpr>,
}

impl<T> std::fmt::Display for Query<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.constraint)?;
        if let Some(ref order_by) = self.order_by {
            write!(f, " ORDER BY {}", order_by)?;
        }
        Ok(())
    }
}

impl<T> From<Constraint<T>> for Query<T> {
    fn from(constraint: Constraint<T>) -> Self {
        Query {
            constraint,
            order_by: None,
        }
    }
}

pub struct Constraint<T> {
    constraint: sql::Expr,
    phantom: std::marker::PhantomData<T>,
}

impl<T> std::fmt::Display for Constraint<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.constraint)
    }
}

impl<T> Constraint<T> {
    fn new(constraint: sql::Expr) -> Self {
        Self {
            constraint,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn and(self, right: Constraint<T>) -> Constraint<T> {
        let constraint = sql::Expr::BinaryOp {
            left: Box::new(self.constraint),
            op: sql::BinaryOperator::And,
            right: Box::new(right.constraint),
        };
        Constraint {
            constraint,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn or(self, right: Constraint<T>) -> Constraint<T> {
        let constraint = sql::Expr::BinaryOp {
            left: Box::new(self.constraint),
            op: sql::BinaryOperator::Or,
            right: Box::new(right.constraint),
        };
        Constraint {
            constraint,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn order_by_asc<F>(self, f: Field<T, F>) -> Query<T> {
        Query {
            constraint: self,
            order_by: Some(sql::OrderByExpr {
                expr: sql::Expr::Identifier(sql::Ident::new(f.field)),
                asc: Some(true),
                nulls_first: None,
            }),
        }
    }

    pub fn order_by_desc<F>(self, f: Field<T, F>) -> Query<T> {
        Query {
            constraint: self,
            order_by: Some(sql::OrderByExpr {
                expr: sql::Expr::Identifier(sql::Ident::new(f.field)),
                asc: Some(false),
                nulls_first: None,
            }),
        }
    }
}

pub struct Field<T, F> {
    field: String,
    phantom: std::marker::PhantomData<(T, F)>,
}

impl<T, F> Field<T, F> {
    pub fn new(field: String) -> Self {
        Field {
            field,
            phantom: std::marker::PhantomData,
        }
    }
}

pub trait ToSQLValue
where
    Self: Sized,
{
    fn to_sql_value(self) -> sql::Value;
}

impl ToSQLValue for String {
    fn to_sql_value(self) -> sql::Value {
        sql::Value::SingleQuotedString(self)
    }
}

impl ToSQLValue for B256 {
    fn to_sql_value(self) -> sql::Value {
        unsafe {
            sql::Value::SingleQuotedByteStringLiteral(
                std::str::from_utf8_unchecked(&self).to_string(),
            )
        }
    }
}

impl ToSQLValue for Bytes {
    fn to_sql_value(self) -> sql::Value {
        unsafe {
            sql::Value::SingleQuotedByteStringLiteral(
                std::str::from_utf8_unchecked(&self).to_string(),
            )
        }
    }
}

impl ToSQLValue for Boolean {
    fn to_sql_value(self) -> sql::Value {
        sql::Value::Boolean(self)
    }
}

impl ToSQLValue for UID {
    fn to_sql_value(self) -> sql::Value {
        sql::Value::SingleQuotedString(self.to_string())
    }
}

macro_rules! impl_number_to_sql_value {
    ($T:ident) => {
        impl ToSQLValue for fuel_indexer_types::scalar::$T {
            fn to_sql_value(self) -> sql::Value {
                sqlparser::test_utils::number(&self.to_string())
            }
        }
    };
}

impl_number_to_sql_value!(I128);
impl_number_to_sql_value!(U128);

impl_number_to_sql_value!(I64);
impl_number_to_sql_value!(U64);

impl_number_to_sql_value!(I32);
impl_number_to_sql_value!(U32);

impl_number_to_sql_value!(I8);
impl_number_to_sql_value!(U8);

impl<T, F: ToSQLValue> Field<T, F> {
    pub fn eq(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::Eq, val)
    }

    pub fn ne(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::NotEq, val)
    }

    pub fn gt(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::Gt, val)
    }

    pub fn ge(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::GtEq, val)
    }

    pub fn lt(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::Lt, val)
    }

    pub fn le(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::LtEq, val)
    }

    fn constraint(self, op: sql::BinaryOperator, val: F) -> Constraint<T> {
        let expr = sql::Expr::BinaryOp {
            left: Box::new(sql::Expr::Identifier(sql::Ident::new(self.field.clone()))),
            op,
            right: Box::new(sql::Expr::Value(val.to_sql_value())),
        };
        Constraint::new(expr)
    }
}

pub struct OptionField<T, F> {
    field: String,
    phantom: std::marker::PhantomData<(T, F)>,
}

impl<T, F> OptionField<T, F> {
    pub fn new(field: String) -> Self {
        OptionField {
            field,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T, F: ToSQLValue> OptionField<T, F> {
    pub fn eq(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::Eq, val)
    }

    pub fn ne(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::NotEq, val)
    }

    pub fn gt(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::Gt, val)
    }

    pub fn ge(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::GtEq, val)
    }

    pub fn lt(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::Lt, val)
    }

    pub fn le(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::LtEq, val)
    }

    pub fn is_null(self) -> Constraint<T> {
        Constraint::new(sql::Expr::IsNull(Box::new(sql::Expr::Identifier(
            sql::Ident::new(self.field),
        ))))
    }

    pub fn is_not_null(self) -> Constraint<T> {
        Constraint::new(sql::Expr::IsNotNull(Box::new(sql::Expr::Identifier(
            sql::Ident::new(self.field),
        ))))
    }

    // Helper function that unwraps the Option converting None to NULL.
    fn constraint(self, op: sql::BinaryOperator, val: F) -> Constraint<T> {
        let expr = sql::Expr::BinaryOp {
            left: Box::new(sql::Expr::Identifier(sql::Ident::new(self.field))),
            op,
            right: Box::new(sql::Expr::Value(val.to_sql_value())),
        };
        Constraint::new(expr)
    }
}

#[no_mangle]
/// Allocation function to be called by an executor in a WASM runtime.
fn alloc_fn(size: u32) -> *const u8 {
    let vec = Vec::with_capacity(size as usize);
    let ptr = vec.as_ptr();

    core::mem::forget(vec);

    ptr
}

#[no_mangle]
/// Deallocation function to be called by an executor in a WASM runtime.
fn dealloc_fn(ptr: *mut u8, len: usize) {
    let _vec = unsafe { Vec::from_raw_parts(ptr, len, len) };
}

#[no_mangle]
/// Immediately terminate WASM execution with the specified error code.
pub fn early_exit(err_code: WasmIndexerError) -> ! {
    unsafe { ff_early_exit(err_code as u32) }
    unreachable!("Expected termination of WASM exetution after a call to ff_early_exit.")
}
