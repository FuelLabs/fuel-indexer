pub mod core;

pub use crate::core::METRICS;
use prometheus::{self, Encoder, Error, TextEncoder};
use thiserror::Error;

pub type MetricsResult<T> = std::result::Result<T, MetricsError>;

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
