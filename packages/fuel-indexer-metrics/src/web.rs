use crate::core::Metric;
use lazy_static::lazy_static;
use prometheus::{self, register_histogram, register_int_counter, Histogram, IntCounter};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Label {
    path: String,
}

lazy_static! {
    static ref REQUEST_TIMING_BUCKETS: Vec<f64> =
        vec![0., 10., 100., 1000., 10000., 50000., 100000., 500000., 1000000.];
}

#[macro_export]
macro_rules! web_metric {
    ($name: ident) => {
        pub struct $name {
            pub requests: IntCounter,
            pub timing: Histogram,
        }

        impl Metric for $name {
            fn init() -> Self {
                Self {
                    requests: register_int_counter!(
                        "Requests",
                        "Number of requests made to endpoint"
                    )
                    .unwrap(),
                    timing: register_histogram!(
                        "RequestTiming",
                        "Time taken to respond to endpoint.",
                        REQUEST_TIMING_BUCKETS.to_vec()
                    )
                    .unwrap(),
                }
            }
        }
    };
}

web_metric!(HealthCheck);
web_metric!(RevertIndexer);
web_metric!(StopIndexer);
web_metric!(RegisterIndexerAssets);
web_metric!(GetNonce);
web_metric!(VerifySignature);
web_metric!(RunQuery);

pub struct Web {
    pub health_check: HealthCheck,
    pub stop_indexer: StopIndexer,
    pub revert_indexer: RevertIndexer,
    pub register_indexer_assets: RegisterIndexerAssets,
    pub get_nonce: GetNonce,
    pub verify_signature: VerifySignature,
    pub run_query: RunQuery,
}

impl Metric for Web {
    fn init() -> Self {
        Self {
            health_check: HealthCheck::init(),
            stop_indexer: StopIndexer::init(),
            revert_indexer: RevertIndexer::init(),
            register_indexer_assets: RegisterIndexerAssets::init(),
            get_nonce: GetNonce::init(),
            verify_signature: VerifySignature::init(),
            run_query: RunQuery::init(),
        }
    }
}
