use crate::{
    api::{ApiError, ApiResult, HttpError},
    models::{Claims, VerifySignatureRequest},
};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::GraphQLRequest;
use async_std::sync::{Arc, RwLock};
use axum::{
    body::Body,
    extract::{multipart::Multipart, Extension, Json, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use fuel_crypto::{Message, Signature};
use fuel_indexer_database::{
    queries,
    types::{IndexAsset, IndexAssetType},
    IndexerConnectionPool,
};
use fuel_indexer_graphql::dynamic::{build_dynamic_schema, execute_query};
use fuel_indexer_lib::{
    config::{auth::AuthenticationStrategy, IndexerConfig},
    defaults,
    utils::{
        AssetReloadRequest, FuelNodeHealthResponse, IndexRevertRequest, IndexStopRequest,
        ServiceRequest, ServiceStatus,
    },
};
use fuel_indexer_schema::db::manager::SchemaManager;
use hyper::Client;
use hyper_rustls::HttpsConnectorBuilder;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use std::{convert::From, str::FromStr, time::Instant};
use tokio::sync::mpsc::Sender;
use tracing::error;

#[cfg(feature = "metrics")]
use fuel_indexer_metrics::encode_metrics_response;

#[cfg(feature = "metrics")]
use http::Request;

pub(crate) async fn query_graph(
    Path((namespace, identifier)): Path<(String, String)>,
    Extension(pool): Extension<IndexerConnectionPool>,
    Extension(manager): Extension<Arc<RwLock<SchemaManager>>>,
    req: GraphQLRequest,
) -> ApiResult<axum::Json<Value>> {
    match manager
        .read()
        .await
        .load_schema(&namespace, &identifier)
        .await
    {
        Ok(schema) => {
            let dynamic_schema = build_dynamic_schema(&schema)?;
            let user_query = req.0.query.clone();
            let response =
                execute_query(req.into_inner(), dynamic_schema, user_query, pool, schema)
                    .await?;
            let data = serde_json::json!({ "data": response });
            Ok(axum::Json(data))
        }
        Err(_e) => Err(ApiError::Http(HttpError::NotFound(format!(
            "The graph '{namespace}.{identifier}' was not found."
        )))),
    }
}

pub(crate) async fn get_fuel_status(config: &IndexerConfig) -> ServiceStatus {
    let https = HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();

    let client = Client::builder().build::<_, hyper::Body>(https);
    match client
        .get(config.to_owned().fuel_node.health_check_uri())
        .await
    {
        Ok(r) => {
            let body_bytes = hyper::body::to_bytes(r.into_body())
                .await
                .unwrap_or_default();

            let fuel_node_health: FuelNodeHealthResponse =
                serde_json::from_slice(&body_bytes).unwrap_or_default();

            ServiceStatus::from(fuel_node_health)
        }
        Err(e) => {
            error!("Failed to fetch fuel /health status: {e}.");
            ServiceStatus::NotOk
        }
    }
}

pub(crate) async fn health_check(
    Extension(config): Extension<IndexerConfig>,
    Extension(pool): Extension<IndexerConnectionPool>,
    Extension(start_time): Extension<Arc<Instant>>,
) -> ApiResult<axum::Json<Value>> {
    let db_status = pool.is_connected().await.unwrap_or(ServiceStatus::NotOk);
    let uptime = start_time.elapsed().as_secs().to_string();
    let fuel_core_status = get_fuel_status(&config).await;

    Ok(Json(json!({
        "fuel_core_status": fuel_core_status,
        "uptime(seconds)": uptime,
        "database_status": db_status,
    })))
}

pub(crate) async fn indexer_status(
    Extension(pool): Extension<IndexerConnectionPool>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<axum::Json<Value>> {
    if claims.is_unauthenticated() {
        return Err(ApiError::Http(HttpError::Unauthorized));
    }

    let mut conn = pool.acquire().await?;

    let indexers: Vec<_> = {
        let indexers = queries::all_registered_indexers(&mut conn).await?;
        if claims.sub().is_empty() {
            indexers
        } else {
            indexers
                .into_iter()
                .filter(|i| i.pubkey.as_ref() == Some(&claims.sub().to_string()))
                .collect()
        }
    };

    let json: serde_json::Value = serde_json::to_value(indexers)?;

    Ok(Json(json!(json)))
}

pub(crate) async fn remove_indexer(
    Path((namespace, identifier)): Path<(String, String)>,
    Extension(tx): Extension<Sender<ServiceRequest>>,
    Extension(pool): Extension<IndexerConnectionPool>,
    Extension(claims): Extension<Claims>,
    Extension(config): Extension<IndexerConfig>,
) -> ApiResult<axum::Json<Value>> {
    if claims.is_unauthenticated() {
        return Err(ApiError::Http(HttpError::Unauthorized));
    }

    let mut conn = pool.acquire().await?;

    queries::start_transaction(&mut conn).await?;

    if config.authentication.enabled {
        queries::indexer_owned_by(&mut conn, &namespace, &identifier, claims.iss())
            .await
            .map_err(|_e| ApiError::Http(HttpError::Unauthorized))?;
    }

    if let Err(e) = queries::remove_indexer(&mut conn, &namespace, &identifier).await {
        error!("Failed to remove Indexer({namespace}.{identifier}): {e}");
        queries::revert_transaction(&mut conn).await?;
        return Err(ApiError::Sqlx(sqlx::Error::RowNotFound));
    }

    queries::commit_transaction(&mut conn).await?;

    tx.send(ServiceRequest::IndexStop(IndexStopRequest {
        namespace,
        identifier,
    }))
    .await?;

    Ok(Json(json!({
        "success": "true"
    })))
}

pub(crate) async fn revert_indexer(
    Path((namespace, identifier)): Path<(String, String)>,
    Extension(tx): Extension<Sender<ServiceRequest>>,
    Extension(pool): Extension<IndexerConnectionPool>,
    Extension(claims): Extension<Claims>,
    Extension(config): Extension<IndexerConfig>,
) -> ApiResult<axum::Json<Value>> {
    if claims.is_unauthenticated() {
        return Err(ApiError::Http(HttpError::Unauthorized));
    }

    let mut conn = pool.acquire().await?;

    if config.authentication.enabled {
        queries::indexer_owned_by(&mut conn, &namespace, &identifier, claims.iss())
            .await
            .map_err(|_e| ApiError::Http(HttpError::Unauthorized))?;
    }

    queries::start_transaction(&mut conn).await?;

    let indexer_id = queries::get_indexer_id(&mut conn, &namespace, &identifier).await?;
    let wasm =
        queries::latest_asset_for_indexer(&mut conn, &indexer_id, IndexAssetType::Wasm)
            .await?;

    if let Err(e) = queries::remove_asset_by_version(
        &mut conn,
        &indexer_id,
        &wasm.version,
        IndexAssetType::Wasm,
    )
    .await
    {
        error!(
            "Could not remove latest WASM asset for Indexer({namespace}.{identifier}): {e}"
        );
        queries::revert_transaction(&mut conn).await?;
        return Err(ApiError::default());
    }

    queries::commit_transaction(&mut conn).await?;

    tx.send(ServiceRequest::IndexRevert(IndexRevertRequest {
        namespace,
        identifier,
    }))
    .await?;

    Ok(Json(json!({
        "success": "true"
    })))
}

pub(crate) async fn register_indexer_assets(
    Path((namespace, identifier)): Path<(String, String)>,
    Extension(tx): Extension<Sender<ServiceRequest>>,
    Extension(schema_manager): Extension<Arc<RwLock<SchemaManager>>>,
    Extension(claims): Extension<Claims>,
    Extension(pool): Extension<IndexerConnectionPool>,
    Extension(config): Extension<IndexerConfig>,
    multipart: Option<Multipart>,
) -> ApiResult<axum::Json<Value>> {
    if claims.is_unauthenticated() {
        return Err(ApiError::Http(HttpError::Unauthorized));
    }

    let mut conn = pool.acquire().await?;

    if config.authentication.enabled {
        queries::indexer_owned_by(&mut conn, &namespace, &identifier, claims.iss())
            .await
            .map_err(|_e| ApiError::Http(HttpError::Unauthorized))?;
    }

    let mut assets: Vec<IndexAsset> = Vec::new();

    if let Some(mut multipart) = multipart {
        queries::start_transaction(&mut conn).await?;

        while let Some(field) = multipart.next_field().await.unwrap() {
            let name = field.name().unwrap_or("").to_string();
            let data = field.bytes().await.unwrap_or_default();
            let asset_type =
                IndexAssetType::from_str(&name).expect("Invalid asset type.");

            let asset: IndexAsset = match asset_type {
                IndexAssetType::Wasm | IndexAssetType::Manifest => {
                    queries::register_indexer_asset(
                        &mut conn,
                        &namespace,
                        &identifier,
                        data.to_vec(),
                        asset_type,
                        Some(claims.sub()),
                    )
                    .await?
                }
                IndexAssetType::Schema => {
                    match queries::register_indexer_asset(
                        &mut conn,
                        &namespace,
                        &identifier,
                        data.to_vec(),
                        IndexAssetType::Schema,
                        Some(claims.sub()),
                    )
                    .await
                    {
                        Ok(result) => {
                            schema_manager
                                .write()
                                .await
                                .new_schema(
                                    &namespace,
                                    &identifier,
                                    &String::from_utf8_lossy(&data),
                                    &mut conn,
                                    // Can't deploy native indexers
                                    false,
                                )
                                .await?;

                            result
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
            };

            assets.push(asset);
        }

        queries::commit_transaction(&mut conn).await?;

        tx.send(ServiceRequest::AssetReload(AssetReloadRequest {
            namespace,
            identifier,
        }))
        .await?;

        return Ok(Json(json!({
            "success": "true",
            "assets": assets,
        })));
    }

    Err(ApiError::default())
}

pub(crate) async fn get_nonce(
    Extension(pool): Extension<IndexerConnectionPool>,
) -> ApiResult<axum::Json<Value>> {
    let mut conn = pool.acquire().await?;
    let nonce = queries::create_nonce(&mut conn).await?;
    Ok(Json(json!(nonce)))
}

pub(crate) async fn verify_signature(
    Extension(config): Extension<IndexerConfig>,
    Extension(pool): Extension<IndexerConnectionPool>,
    Json(payload): Json<VerifySignatureRequest>,
) -> ApiResult<axum::Json<Value>> {
    if config.authentication.enabled {
        let mut conn = pool.acquire().await?;
        match config.authentication.strategy {
            Some(AuthenticationStrategy::JWT) => {
                let nonce = queries::get_nonce(&mut conn, &payload.message).await?;

                if nonce.is_expired() {
                    return Err(ApiError::Http(HttpError::Unauthorized));
                }

                let buff: [u8; 64] = hex::decode(&payload.signature)?
                    .try_into()
                    .unwrap_or([0u8; 64]);
                let sig = Signature::from_bytes(buff);
                let msg = Message::new(payload.message);
                let pk = sig.recover(&msg)?;

                let claims = Claims::new(
                    pk.to_string(),
                    config.authentication.jwt_issuer.unwrap_or_default(),
                    config
                        .authentication
                        .jwt_expiry
                        .unwrap_or(defaults::JWT_EXPIRY_SECS),
                );

                if let Err(e) = sig.verify(&pk, &msg) {
                    error!("Failed to verify signature: {e}.");
                    return Err(ApiError::FuelCrypto(e));
                }

                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(
                        config
                            .authentication
                            .jwt_secret
                            .unwrap_or_default()
                            .as_ref(),
                    ),
                )?;

                queries::delete_nonce(&mut conn, &nonce).await?;

                return Ok(Json(json!({ "token": token })));
            }
            _ => {
                error!("Unsupported authentication strategy.");
                unimplemented!();
            }
        }
    }
    unreachable!();
}

pub async fn gql_playground(
    Path((namespace, identifier)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let html = playground_source(
        GraphQLPlaygroundConfig::new(&format!("/api/graph/{namespace}/{identifier}"))
            .with_setting("schema.polling.enable", false),
    );

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(html))?;

    Ok(response)
}

#[cfg(feature = "metrics")]
pub async fn get_metrics(_req: Request<Body>) -> impl IntoResponse {
    encode_metrics_response()
}
