use crate::{
    api::{ApiError, ApiResult, HttpError},
    models::{Claims, SqlQuery, VerifySignatureRequest},
    sql::SqlQueryValidator,
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
    types::{IndexerAsset, IndexerAssetType},
    IndexerConnectionPool,
};
use fuel_indexer_graphql::dynamic::{build_dynamic_schema, execute_query};
use fuel_indexer_lib::{
    config::{auth::AuthenticationStrategy, IndexerConfig},
    defaults,
    graphql::GraphQLSchema,
    utils::{
        FuelClientHealthResponse, ReloadRequest, ServiceRequest, ServiceStatus,
        StopRequest,
    },
    ExecutionSource,
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

/// Given an indexer namespace and identifier, return the results for the given
/// `GraphQLRequest`.
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

/// Return the `ServiceStatus` for the Fuel client.
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

            let clienth_health: FuelClientHealthResponse =
                serde_json::from_slice(&body_bytes).unwrap_or_default();

            ServiceStatus::from(clienth_health)
        }
        Err(e) => {
            error!("Failed to fetch Fuel client health status: {e}.");
            ServiceStatus::NotOk
        }
    }
}

/// Return a JSON payload with the health status of various components, including
/// the fuel client, the database, and the uptime of the service.
pub(crate) async fn health_check(
    Extension(config): Extension<IndexerConfig>,
    Extension(pool): Extension<IndexerConnectionPool>,
    Extension(start_time): Extension<Arc<Instant>>,
) -> ApiResult<axum::Json<Value>> {
    let db_status = pool.is_connected().await.unwrap_or(ServiceStatus::NotOk);
    let uptime = start_time.elapsed().as_secs().to_string();
    let client_status = get_fuel_status(&config).await;

    Ok(Json(json!({
        "client_status": client_status,
        "uptime": uptime,
        "database_status": db_status,
    })))
}

/// Return a JSON payload containing the status of a given indexer, or set of indexers.
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

/// Given an indexer namespace and identifier, remove the indexer from the database
/// and send a `ServiceRequest::Stop` to the service for this indexer.
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
        queries::indexer_owned_by(&mut conn, &namespace, &identifier, claims.sub())
            .await
            .map_err(|_e| ApiError::Http(HttpError::Unauthorized))?;
    }

    if let Err(e) =
        queries::remove_indexer(&mut conn, &namespace, &identifier, false).await
    {
        error!("Failed to remove Indexer({namespace}.{identifier}): {e}");
        queries::revert_transaction(&mut conn).await?;
        return Err(ApiError::Sqlx(sqlx::Error::RowNotFound));
    }

    queries::commit_transaction(&mut conn).await?;

    tx.send(ServiceRequest::Stop(StopRequest {
        namespace,
        identifier,
        notify: None,
    }))
    .await?;

    Ok(Json(json!({
        "success": "true"
    })))
}

/// Given an indexer namespace and identifier, register the indexer in the database, and
/// send a `ServiceRequest::Reload` to the service for this indexer.
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
    let mut assets: Vec<IndexerAsset> = Vec::new();

    if let Some(mut multipart) = multipart {
        queries::start_transaction(&mut conn).await?;

        let mut replace_indexer: bool = false;
        let mut remove_data: bool = false;
        let mut asset_bytes: Vec<(IndexerAssetType, hyper::body::Bytes)> = Vec::new();

        while let Some(field) = multipart.next_field().await.unwrap() {
            let name = field.name().unwrap_or("").to_string();
            let data = field.bytes().await.unwrap_or_default();
            match name.as_str() {
                "replace_indexer" => {
                    println!("FOO {:?}", data.to_owned());
                    replace_indexer = std::str::from_utf8(&data.to_owned())
                        .unwrap()
                        .parse()
                        .unwrap();
                    continue;
                }
                "remove_data" => {
                    remove_data = std::str::from_utf8(&data.to_owned())
                        .unwrap()
                        .parse()
                        .unwrap();
                    continue;
                }
                name => {
                    let asset_type =
                        IndexerAssetType::from_str(&name).expect("Invalid asset type.");
                    asset_bytes.push((asset_type, data))
                }
            };
        }

        let indexer_exists = queries::get_indexer_id(&mut conn, &namespace, &identifier)
            .await
            .is_ok();

        if indexer_exists {
            // --replace-indexer is only allowed if it has also been enabled at
            // the fuel-indexer service level
            if config.replace_indexer && replace_indexer {
                let (sender, receiver) = futures::channel::oneshot::channel();
                tx.send(ServiceRequest::Stop(StopRequest {
                    namespace: namespace.clone(),
                    identifier: identifier.clone(),
                    notify: Some(sender),
                }))
                .await?;

                // Since we remove and recreate indexer tables, we need to wait
                // for the idexer to stop to ensure the old indexer does not
                // write any data to the newly created tables.
                if let Err(_) = receiver.await {
                    return Err(ApiError::Http(HttpError::InternalServer));
                }

                if let Err(e) =
                    queries::remove_indexer(&mut conn, &namespace, &identifier, remove_data)
                        .await
                {
                    error!("Failed to remove Indexer({namespace}.{identifier}): {e}");
                    queries::revert_transaction(&mut conn).await?;
                    return Err(e.into());
                }
            } else {
                error!("Indexer({namespace}.{identifier}) already exists.");
                queries::revert_transaction(&mut conn).await?;
                return Err(ApiError::Http(HttpError::Conflict(format!(
                    "Indexer({namespace}.{identifier}) already exists"
                ))));
            }
        }

        for (asset_type, data) in asset_bytes.iter() {
            match asset_type {
                IndexerAssetType::Wasm | IndexerAssetType::Manifest => {
                    match queries::register_indexer_asset(
                        &mut conn,
                        &namespace,
                        &identifier,
                        data.to_vec(),
                        asset_type.clone(),
                        Some(claims.sub()),
                    )
                    .await
                    {
                        Ok(result) => {
                            assets.push(result);
                        }
                        Err(e) => {
                            let _res = queries::revert_transaction(&mut conn).await?;
                            return Err(e.into());
                        }
                    }
                }
                IndexerAssetType::Schema => {
                    match queries::register_indexer_asset(
                        &mut conn,
                        &namespace,
                        &identifier,
                        data.to_vec(),
                        IndexerAssetType::Schema,
                        Some(claims.sub()),
                    )
                    .await
                    {
                        Ok(result) => {
                            let schema = GraphQLSchema::new(
                                String::from_utf8_lossy(&data).to_string(),
                            );
                            if remove_data {
                                match schema_manager
                                    .write()
                                    .await
                                    .new_schema(
                                        &namespace,
                                        &identifier,
                                        schema,
                                        // Only WASM can be sent over the web.
                                        ExecutionSource::Wasm,
                                        &mut conn,
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        assets.push(result);
                                    }
                                    Err(e) => {
                                        let _res = queries::revert_transaction(&mut conn)
                                            .await?;
                                        return Err(e.into());
                                    }
                                }
                            } else {
                                assets.push(result);
                            }
                        }
                        Err(e) => {
                            let _res = queries::revert_transaction(&mut conn).await?;
                            return Err(e.into());
                        }
                    }
                }
            }
        }

        queries::commit_transaction(&mut conn).await?;

        tx.send(ServiceRequest::Reload(ReloadRequest {
            namespace,
            identifier,
            remove_data,
            replace_indexer,
        }))
        .await?;

        return Ok(Json(json!({
            "success": "true",
            "assets": assets,
        })));
    }

    Err(ApiError::default())
}

/// Return a `Nonce` to be used for authentication.
pub(crate) async fn get_nonce(
    Extension(pool): Extension<IndexerConnectionPool>,
) -> ApiResult<axum::Json<Value>> {
    let mut conn = pool.acquire().await?;
    let nonce = queries::create_nonce(&mut conn).await?;
    Ok(Json(json!(nonce)))
}

/// Given a message and signature, verify the signature and return a JWT token for authentication.
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

/// Endpoint for the GraphQL playground.
///
/// This is route just produces/creates the GraphQL playground, the actual queries
/// submitted from the playground are still handled by `uses::query_graph`.
pub async fn graphql_playground(
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

/// Return a response containing various Prometheus metrics for the service.
#[cfg(feature = "metrics")]
pub async fn get_metrics(_req: Request<Body>) -> impl IntoResponse {
    encode_metrics_response()
}

/// Return the results from a validated, arbitrary SQL query.
pub async fn sql_query(
    Path((_namespace, _identifier)): Path<(String, String)>,
    Extension(claims): Extension<Claims>,
    Extension(pool): Extension<IndexerConnectionPool>,
    Json(query): Json<SqlQuery>,
) -> ApiResult<axum::Json<Value>> {
    if claims.is_unauthenticated() {
        return Err(ApiError::Http(HttpError::Unauthorized));
    }
    let SqlQuery { query } = query;
    SqlQueryValidator::validate_sql_query(&query)?;
    let mut conn = pool.acquire().await?;
    let result = queries::run_query(&mut conn, query).await?;
    Ok(Json(json!({ "data": result })))
}
