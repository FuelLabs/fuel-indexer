use crate::core::Metric;
use lazy_static::lazy_static;
use prometheus::{self, register_histogram, register_int_counter, Histogram, IntCounter};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Label {
    path: String,
}

lazy_static! {
    static ref REQUEST_TIMING_BUCKETS: Vec<f64> =
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];
}

pub struct HealthCheckEndpoint {
    pub requests: IntCounter,
    pub timing: Histogram,
}

impl Metric for HealthCheckEndpoint {
    fn init() -> Self {
        HealthCheckEndpoint {
            requests: register_int_counter!(
                "Requests",
                "Number of requests made to /healthcheck."
            )
            .unwrap(),
            timing: register_histogram!(
                "RequestTiming",
                "Time taken to respond to /healthcheck.",
                REQUEST_TIMING_BUCKETS.to_vec()
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
