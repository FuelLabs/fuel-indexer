use crate::{core::Metric, queries::PostgreQueries};
use prometheus::{self, register_int_counter, IntCounter};

pub struct Database {
    pub write_ops: IntCounter,
    pub read_ops: IntCounter,
    pub bytes_written: IntCounter,
    pub bytes_read: IntCounter,
    pub postgres: PostgreQueries,
}

impl Metric for Database {
    fn init() -> Self {
        Self {
            postgres: PostgreQueries::init(),
            write_ops: register_int_counter!("write_ops", "Count of write operations.")
                .unwrap(),
            read_ops: register_int_counter!("read_ops", "Count of read operations.")
                .unwrap(),
            bytes_written: register_int_counter!(
                "bytes_written",
                "Total bytes written to the database."
            )
            .unwrap(),
            bytes_read: register_int_counter!(
                "bytes_read",
                "Total bytes read from the database."
            )
            .unwrap(),
        }
    }
}
