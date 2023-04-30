#![deny(unused_crate_dependencies)]

pub mod core;

use crate::core::{Metric, Metrics};
use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use lazy_static::lazy_static;
use prometheus_client::encoding::text::encode;

lazy_static! {
    pub static ref METRICS: Metrics = Metrics::init();
}

pub fn encode_metrics_response() -> impl IntoResponse {
    let mut encoded = String::new();

    if encode(&mut encoded, &METRICS.web.registry).is_err() {
        return error_body();
    }

    if encode(&mut encoded, &METRICS.db.postgres.registry).is_err() {
        return error_body();
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(encoded))
        .unwrap()
}

fn error_body() -> Response<Body> {
    Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .body(Body::from("unavailable"))
        .unwrap()
}
