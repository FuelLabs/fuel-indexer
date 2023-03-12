use axum::http::Request;
use fuel_indexer_lib::config::{
    auth::{AuthenticationStrategy, Claims},
    IndexerConfig,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::error;

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

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let header = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok());

        let header = if let Some(header) = header {
            header
        } else {
            unimplemented!();
        };

        let config = &self.state.config;

        if config.authentication.enabled {
            match &config.authentication.strategy {
                Some(AuthenticationStrategy::JWT) => {
                    let secret =
                        config.authentication.jwt_secret.clone().unwrap_or_default();
                    match decode::<Claims>(
                        header,
                        &DecodingKey::from_secret(secret.as_bytes()),
                        &Validation::default(),
                    ) {
                        Ok(claims) => {
                            req.extensions_mut().insert(claims);
                            self.inner.call(req)
                        }
                        Err(e) => {
                            error!("Failed to decode claims: {e}");
                            unimplemented!();
                        }
                    }
                }
                _ => unimplemented!(),
            }
        } else {
            self.inner.call(req)
        }
    }
}
