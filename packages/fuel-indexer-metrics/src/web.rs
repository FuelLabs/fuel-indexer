use crate::core::Metric;
use lazy_static::lazy_static;
use prometheus::{self, register_int_counter, IntCounter};
use prometheus_client::{
    encoding::EncodeLabelSet,
    metrics::{family::Family, histogram::Histogram},
    registry::Registry,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct Label {
    path: String,
}

lazy_static! {
    static ref GRAPHQL_HISTOGRAM_BUCKETS: Vec<f64> =
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];
}

pub struct GraphQLApi {
    pub registry: Registry,
    requests: Family<Label, Histogram>,
}

impl Metric for GraphQLApi {
    fn init() -> Self {
        let mut registry = Registry::default();
        let requests = Family::<Label, Histogram>::new_with_constructor(|| {
            Histogram::new(GRAPHQL_HISTOGRAM_BUCKETS.iter().cloned())
        });
        registry.register("graphql_request_duration_seconds", "", requests.clone());
        Self { registry, requests }
    }
}

impl GraphQLApi {
    pub fn graphql_observe(&self, query: &str, time: f64) {
        let histogram = self.requests.get_or_create(&Label {
            path: query.to_string(),
        });
        histogram.observe(time);
    }
}

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
    pub graphql: GraphQLApi,
}

impl Metric for Web {
    fn init() -> Self {
        Self {
            health: HealthCheckEndpoint::init(),
            graphql: GraphQLApi::init(),
        }
    }
}
