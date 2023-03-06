#![deny(unused_crate_dependencies)]

pub mod core;
pub mod queries;

use crate::core::{Metric, Metrics};
use lazy_static::lazy_static;
use prometheus::{self, Encoder, Error, TextEncoder};
use thiserror::Error;

lazy_static! {
    pub static ref METRICS: Metrics = Metrics::init();
}

pub(crate) type MetricsResult<T> = std::result::Result<T, MetricsError>;

#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("Encoding metrics error: {0:#?}")]
    MetricsEncoding(#[from] Error),
}

pub fn encode_metrics_response() -> MetricsResult<(Vec<u8>, String)> {
    let mut buff = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    match encoder.encode(&metric_families, &mut buff) {
        Ok(_) => Ok((buff, encoder.format_type().to_owned())),
        Err(e) => Err(MetricsError::MetricsEncoding(e)),
    }
}
