use crate::utils::metrics_label_for_request;
use axum::{body::Body, http::Request, response::Response};
use fuel_indexer_metrics::METRICS;
use futures_util::future::BoxFuture;
use std::task::{Context, Poll};
use std::time::Instant;
use tower::{Layer, Service};

#[derive(Clone, Default)]
struct MiddlewareState;

#[derive(Clone, Default)]
pub struct MetricsMiddleware {
    #[allow(unused)]
    state: MiddlewareState,
}

impl<S> Layer<S> for MetricsMiddleware {
    type Service = MetricsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsService {
            inner,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct MetricsService<S> {
    inner: S,
    #[allow(unused)]
    state: MiddlewareState,
}

impl<S> Service<Request<Body>> for MetricsService<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let start_time = Instant::now();
        let label = metrics_label_for_request(&req);

        let fut = self.inner.call(req);
        Box::pin(async move {
            let resp: Response = fut.await?;

            METRICS
                .web
                .record(&label, start_time.elapsed().as_millis() as f64);
            Ok(resp)
        })
    }
}
