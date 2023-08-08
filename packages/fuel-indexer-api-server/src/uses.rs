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

    // Allways remove data when removing an indexer
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

    let multipart = multipart.ok_or_else(|| ApiError::default())?;

    let (replace_indexer, remove_data, asset_bytes) =
        parse_register_indexer_multipart(multipart).await?;

    queries::start_transaction(&mut conn).await?;

    let result = register_indexer_assets_transaction(
        &mut conn,
        schema_manager.clone(),
        tx.clone(),
        config,
        &namespace,
        &identifier,
        claims.sub(),
        replace_indexer,
        remove_data,
        asset_bytes,
    )
    .await;

    match result {
        Ok(assets) => {
            queries::commit_transaction(&mut conn).await?;

            tx.send(ServiceRequest::Reload(ReloadRequest {
                namespace,
                identifier,
            }))
            .await?;

            return Ok(Json(json!({
                "success": "true",
                "assets": assets,
            })));
        }
        Err(e) => {
            queries::revert_transaction(&mut conn).await?;
            return Err(e);
        }
    }
}

async fn register_indexer_assets_transaction(
    conn: &mut fuel_indexer_database::IndexerConnection,
    schema_manager: Arc<RwLock<SchemaManager>>,
    tx: Sender<ServiceRequest>,
    config: IndexerConfig,
    namespace: &str,
    identifier: &str,
    pubkey: &str,
    replace_indexer: bool,
    remove_data: bool,
    asset_bytes: Vec<(IndexerAssetType, Vec<u8>)>,
) -> ApiResult<Vec<IndexerAsset>> {
    let mut assets: Vec<IndexerAsset> = Vec::new();

    let indexer_id = queries::get_indexer_id(conn, &namespace, &identifier).await;

    // If the indexer already exists, check that the schema has not changed.
    if let Ok(indexer_id) = indexer_id {
        // And we're not removing data
        if !remove_data {
            for (asset_type, data) in asset_bytes.iter() {
                if *asset_type == IndexerAssetType::Schema {
                    // The schema must be the same. This query returns an asset
                    // if the bytes match. If it returns None (and the indexer
                    // exists), it means that its schema is different.
                    let schema = {
                        let content = String::from_utf8(data.to_vec())
                            .map_err(|e| {
                                ApiError::OtherError(format!("Invalid schema: {}", e))
                            })?
                            .to_string();
                        GraphQLSchema::new(content)
                    };
                    if queries::asset_already_exists(
                        conn,
                        &IndexerAssetType::Schema,
                        &Vec::<u8>::from(&schema),
                        &indexer_id,
                    )
                    .await?
                    .is_none()
                    {
                        return Err(ApiError::Http(HttpError::Conflict(format!(
                            "Indexer({namespace}.{identifier})'s schema has changed. Use --replace-indexer --remove-data to replace the indexer and the indexed data."
                        ))));
                    }
                }
            }
        }
    }

    if !config.replace_indexer && replace_indexer {
        error!("Failed to replace Indexer({namespace}.{identifier}): replacing an indexer is not enabled.");
        return Err(ApiError::Http(HttpError::Conflict(format!(
            "Failed to replace Indexer({namespace}.{identifier}): replacing an indexer is not enabled."
        ))));
    }

    if indexer_id.is_ok() {
        // --replace-indexer is only allowed if it has also been enabled at
        // the fuel-indexer service level
        if config.replace_indexer && replace_indexer {
            tx.send(ServiceRequest::Stop(StopRequest {
                namespace: namespace.to_owned(),
                identifier: identifier.to_owned(),
            }))
            .await?;

            queries::remove_indexer(conn, &namespace, &identifier, remove_data)
                .await
                .map_err(|e| {
                    error!("Failed to remove Indexer({namespace}.{identifier}): {e}");
                    e
                })?
        } else {
            error!("Indexer({namespace}.{identifier}) already exists.");
            return Err(ApiError::Http(HttpError::Conflict(format!(
                "Indexer({namespace}.{identifier}) already exists"
            ))));
        }
    }

    for (asset_type, data) in asset_bytes.iter() {
        match asset_type {
            IndexerAssetType::Wasm | IndexerAssetType::Manifest => {
                let result = queries::register_indexer_asset(
                    conn,
                    &namespace,
                    &identifier,
                    data.to_vec(),
                    asset_type.to_owned(),
                    Some(pubkey),
                )
                .await?;

                assets.push(result);
            }
            IndexerAssetType::Schema => {
                let asset = queries::register_indexer_asset(
                    conn,
                    &namespace,
                    &identifier,
                    data.to_vec(),
                    IndexerAssetType::Schema,
                    Some(pubkey),
                )
                .await?;

                let schema =
                    GraphQLSchema::new(String::from_utf8_lossy(&data).to_string());

                schema_manager
                    .write()
                    .await
                    .new_schema(
                        &namespace,
                        &identifier,
                        schema,
                        // Only WASM can be sent over the web.
                        ExecutionSource::Wasm,
                        conn,
                    )
                    .await?;

                assets.push(asset);
            }
        }
    }

    Ok(assets)
}

async fn parse_register_indexer_multipart(
    mut multipart: Multipart,
) -> ApiResult<(bool, bool, Vec<(IndexerAssetType, Vec<u8>)>)> {
    let mut replace_indexer: bool = false;
    let mut remove_data: bool = false;
    let mut assets: Vec<(IndexerAssetType, Vec<u8>)> = vec![];

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        let data = field.bytes().await.unwrap_or_default();
        match name.as_str() {
            "replace_indexer" => {
                replace_indexer = std::str::from_utf8(&data.to_owned())
                    .map_err(|e| ApiError::OtherError(e.to_string()))?
                    .parse::<bool>()
                    .map_err(|e| ApiError::OtherError(e.to_string()))?;
                continue;
            }
            "remove_data" => {
                remove_data = std::str::from_utf8(&data.to_owned())
                    .map_err(|e| ApiError::OtherError(e.to_string()))?
                    .parse::<bool>()
                    .map_err(|e| ApiError::OtherError(e.to_string()))?;
                continue;
            }
            name => {
                let asset_type = IndexerAssetType::from_str(name)?;
                assets.push((asset_type, data.to_vec()));
            }
        };
    }

    Ok((replace_indexer, remove_data, assets))
}

// async fn check_schema_not_changed(
//     conn: fuel_indexer_database::IndexerConnection,
//     indexer_id: i64,
//     assets: Vec<(IndexerAssetType, Vec<u8>)>,
// ) -> ApiResult<()> {
//     // Check that the schema has not changed.
//     for (asset_type, data) in assets.iter() {
//         if *asset_type == IndexerAssetType::Schema {
//             // The schema must be the same. This query returns an
//             // asset if the bytes match. If it returns None (and the
//             // indexer exists), it means that its schema is
//             // different.
//             let schema = {
//                 let content = String::from_utf8(data.to_vec())
//                     .map_err(|e| ApiError::OtherError(format!("Invalid schema: {}", e)))?
//                     .to_string();
//                 GraphQLSchema::new(content)
//             };
//             if queries::asset_already_exists(
//                 &mut conn,
//                 &IndexerAssetType::Schema,
//                 &Vec::<u8>::from(&schema),
//                 &indexer_id,
//             )
//             .await?
//             .is_none()
//             {
//                 queries::revert_transaction(&mut conn).await?;
//                 return Err(ApiError::Http(HttpError::Conflict(format!(
//                     "Indexer({namespace}.{identifier})'s schema has changed. Use --replace-indexer --remove-data to replace the indexer and the indexed data."
//                 ))));
//             }
//         }
//     }

//     Ok(())
// }

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
