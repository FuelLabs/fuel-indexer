use axum::http::{Request, StatusCode};
use fuel_indexer_lib::config::{auth::AuthenticationStrategy, IndexerConfig};
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
struct MiddlewareState {
    config: IndexerConfig,
}

#[derive(Clone)]
pub struct AuthenticationMiddleware {
    state: MiddlewareState,
}

impl From<&IndexerConfig> for AuthenticationMiddleware {
    fn from(config: &IndexerConfig) -> Self {
        Self {
            state: MiddlewareState {
                config: config.clone(),
            },
        }
    }
}

impl<S> Layer<S> for AuthenticationMiddleware {
    type Service = AuthenticationService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthenticationService {
            inner,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthenticationService<S> {
    inner: S,
    state: MiddlewareState,
}

impl<S, B> Service<Request<B>> for AuthenticationService<S>
where
    S: Service<Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let _header = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok());

        let config = &self.state.config;

        if config.authentication.enabled {
            match &config.authentication.strategy {
                Some(AuthenticationStrategy::JWT) => {
                    println!("\n>>> I'm DOING JWT STUFF.\n");
                    self.inner.call(req)
                }
                _ => unimplemented!(),
            }
        } else {
            self.inner.call(req)
        }
    }
}
