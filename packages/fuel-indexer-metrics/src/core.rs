use crate::queries::PostgreQueries;
use prometheus::{self, register_int_counter, IntCounter};

pub trait Metric {
    fn init() -> Self;
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct HealthCheckEndpoint {
    pub requests: IntCounter,
}

impl Metric for HealthCheckEndpoint {
    fn init() -> Self {
        HealthCheckEndpoint {
            requests: register_int_counter!(
                "Requests",
                "Number of requests made to /healthcheck."
            )
            .unwrap(),
        }
    }
}

pub struct Web {
    pub health: HealthCheckEndpoint,
}

impl Metric for Web {
    fn init() -> Self {
        Self {
            health: HealthCheckEndpoint::init(),
        }
    }
}

pub struct Metrics {
    pub web: Web,
    pub db: Database,
}

impl Metric for Metrics {
    fn init() -> Self {
        Self {
            web: Web::init(),
            db: Database::init(),
        }
    }
}
