use crate::core::Metric;
#[allow(unused)]
use prometheus::{self, register_int_counter, Counter, Encoder, IntCounter, TextEncoder};

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
