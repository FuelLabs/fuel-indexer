use lazy_static::lazy_static;
use prometheus::{self, register_int_counter, IntCounter};
use prometheus_client::{
    encoding::EncodeLabelSet,
    metrics::{family::Family, histogram::Histogram},
    registry::Registry,
};

lazy_static! {
    pub static ref TIMING_HISTOGRAM_BUCKETS: Vec<f64> =
        vec![0., 10., 100., 1000., 10000., 50000., 100000., 500000., 1000000.];
}

pub trait Metric {
    fn init() -> Self;
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct Label {
    path: String,
}

pub struct Postgres {
    pub registry: Registry,
    requests: Family<Label, Histogram>,
}

impl Metric for Postgres {
    fn init() -> Self {
        let mut registry = Registry::default();
        let requests = Family::<Label, Histogram>::new_with_constructor(|| {
            Histogram::new(TIMING_HISTOGRAM_BUCKETS.iter().cloned())
        });
        registry.register("postgres_operation_duration", "", requests.clone());

        Self { registry, requests }
    }
}

impl Postgres {
    pub fn record(&self, query: &str, time: f64) {
        let histogram = self.requests.get_or_create(&Label {
            path: query.to_string(),
        });
        histogram.observe(time);
    }
}

pub struct Database {
    pub write_ops: IntCounter,
    pub read_ops: IntCounter,
    pub bytes_written: IntCounter,
    pub bytes_read: IntCounter,
    pub postgres: Postgres,
}

impl Metric for Database {
    fn init() -> Self {
        Self {
            postgres: Postgres::init(),
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

pub struct Web {
    pub registry: Registry,
    requests: Family<Label, Histogram>,
}

impl Metric for Web {
    fn init() -> Self {
        let mut registry = Registry::default();
        let requests = Family::<Label, Histogram>::new_with_constructor(|| {
            Histogram::new(TIMING_HISTOGRAM_BUCKETS.iter().cloned())
        });
        registry.register("web_request_duration", "", requests.clone());
        Self { registry, requests }
    }
}

impl Web {
    pub fn record(&self, query: &str, time: f64) {
        let histogram = self.requests.get_or_create(&Label {
            path: query.to_string(),
        });
        histogram.observe(time);
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
