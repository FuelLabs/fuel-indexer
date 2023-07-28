#![deny(unused_crate_dependencies)]

pub mod api;
pub mod cli;
pub(crate) mod commands;
pub(crate) mod middleware;
pub(crate) mod models;
pub(crate) mod sql;
mod uses;

pub mod utils {
    use axum::body::Body;
    use http::Request;

    pub fn metrics_label_for_request(req: &Request<Body>) -> String {
        let path = req.uri().path();
        let method = req.method().as_str();

        let path = if path.starts_with("/api/graph") {
            "api/graph".to_string()
        } else if path.starts_with("api/index") {
            "api/index".to_string()
        } else if path.starts_with("api/playground") {
            "api/playground".to_string()
        } else {
            path.to_string()
        };

        format!("{method}{path}")
    }
}
