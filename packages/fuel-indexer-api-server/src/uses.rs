use crate::{
    api::{ApiError, ApiResult, HttpError},
    models::{QueryResponse, VerifySignatureRequest},
};
use async_graphql::{
    dynamic::Schema as DynamicSchema, dynamic::*, Value as DynamicValue,
};
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
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
use fuel_indexer_graphql::graphql::GraphqlQueryBuilder;
use fuel_indexer_lib::{
    config::{
        auth::{AuthenticationStrategy, Claims},
        IndexerConfig,
    },
    defaults,
    utils::{
        AssetReloadRequest, FuelNodeHealthResponse, IndexRevertRequest, IndexStopRequest,
        ServiceRequest, ServiceStatus,
    },
};
use fuel_indexer_schema::db::{manager::SchemaManager, tables::Schema};
use hyper::Client;
use hyper_rustls::HttpsConnectorBuilder;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use std::{
    convert::From,
    str::FromStr,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc::Sender;
use tracing::{error};

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
        Ok(schema) => match run_query(req.into_inner().query, schema, &pool).await {
            Ok(query_res) => Ok(axum::Json(query_res)),
            Err(e) => {
                error!("query_graph error: {e}");
                Err(e)
            }
        },
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

pub(crate) async fn stop_indexer(
    Path((namespace, identifier)): Path<(String, String)>,
    Extension(tx): Extension<Sender<ServiceRequest>>,
    Extension(pool): Extension<IndexerConnectionPool>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<axum::Json<Value>> {
    if claims.is_unauthenticated() {
        return Err(ApiError::Http(HttpError::Unauthorized));
    }

    let mut conn = pool.acquire().await?;

    let _ = queries::start_transaction(&mut conn).await?;

    if let Err(e) = queries::remove_indexer(&mut conn, &namespace, &identifier).await {
        queries::revert_transaction(&mut conn).await?;

        error!("Failed to remove Indexer({namespace}.{identifier}): {e}");

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
) -> ApiResult<axum::Json<Value>> {
    if claims.is_unauthenticated() {
        return Err(ApiError::Http(HttpError::Unauthorized));
    }

    let mut conn = pool.acquire().await?;
    let asset = queries::penultimate_asset_for_index(
        &mut conn,
        &namespace,
        &identifier,
        IndexAssetType::Wasm,
    )
    .await?;

    tx.send(ServiceRequest::IndexRevert(IndexRevertRequest {
        penultimate_asset_id: asset.id,
        penultimate_asset_bytes: asset.bytes,
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
    multipart: Option<Multipart>,
) -> ApiResult<axum::Json<Value>> {
    if claims.is_unauthenticated() {
        return Err(ApiError::Http(HttpError::Unauthorized));
    }

    let mut assets: Vec<IndexAsset> = Vec::new();

    if let Some(mut multipart) = multipart {
        let mut conn = pool.acquire().await?;

        let _ = queries::start_transaction(&mut conn).await?;

        while let Some(field) = multipart.next_field().await.unwrap() {
            let name = field.name().unwrap_or("").to_string();
            let data = field.bytes().await.unwrap_or_default();
            let asset_type =
                IndexAssetType::from_str(&name).expect("Invalid asset type.");

            let asset: IndexAsset = match asset_type {
                IndexAssetType::Wasm | IndexAssetType::Manifest => {
                    queries::register_index_asset(
                        &mut conn,
                        &namespace,
                        &identifier,
                        data.to_vec(),
                        asset_type,
                        Some(&claims.sub),
                    )
                    .await?
                }
                IndexAssetType::Schema => {
                    match queries::register_index_asset(
                        &mut conn,
                        &namespace,
                        &identifier,
                        data.to_vec(),
                        IndexAssetType::Schema,
                        Some(&claims.sub),
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

        let _ = queries::commit_transaction(&mut conn).await?;

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

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as usize;

                let claims = Claims {
                    sub: pk.to_string(),
                    iss: config.authentication.jwt_issuer.unwrap_or_default(),
                    iat: now,
                    exp: now
                        + config
                            .authentication
                            .jwt_expiry
                            .unwrap_or(defaults::JWT_EXPIRY_SECS),
                };

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

pub async fn run_query(
    query: String,
    schema: Schema,
    pool: &IndexerConnectionPool,
) -> ApiResult<Value> {
    let builder = GraphqlQueryBuilder::new(&schema, &query)?;
    let query = builder.build()?;

    let queries = query.as_sql(&schema, pool.database_type())?.join(";\n");

    let mut conn = pool.acquire().await?;

    match queries::run_query(&mut conn, queries).await {
        Ok(ans) => {
            let ans_json: Value = serde_json::from_value(ans)?;

            // If the response is paginated, remove the array wrapping.
            if ans_json[0].get("page_info").is_some() {
                Ok(serde_json::json!(QueryResponse {
                    data: ans_json[0].clone()
                }))
            } else {
                Ok(serde_json::json!(QueryResponse { data: ans_json }))
            }
        }
        Err(e) => {
            error!("Error querying database: {e}.");
            Err(e.into())
        }
    }
}

pub async fn gql_playground(
    Path((namespace, identifier)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let html = playground_source(
        GraphQLPlaygroundConfig::new(&format!("/api/graph/{namespace}/{identifier}"))
            .with_setting("scehma.polling.enable", false),
    );

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(html))?;

    Ok(response)
}

#[cfg(feature = "metrics")]
pub async fn metrics(_req: Request<Body>) -> impl IntoResponse {
    encode_metrics_response()
}

//
// Async GraphQL code
//
pub async fn gql_playground_next(
    Path((namespace, identifier)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let html = playground_source(
        GraphQLPlaygroundConfig::new(&format!("/api/next/{namespace}/{identifier}"))
            .with_setting("schema.polling.enable", true),
    );

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(html))?;

    Ok(response)
}



pub(crate) async fn next_query_graph(
    Path((namespace, identifier)): Path<(String, String)>,
    Extension(pool): Extension<IndexerConnectionPool>,
    Extension(manager): Extension<Arc<RwLock<SchemaManager>>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    println!("Start building schema");
    let schema = manager
        .read()
        .await
        .load_schema(&namespace, &identifier)
        .await
        .unwrap();
    let req_ref = req.into_inner();
    let query = &req_ref.query;
    let dy_schema: DynamicSchema =
        build_schema(query.clone(), schema, pool).await.unwrap();

    dy_schema.execute(req_ref).await.into()
}

async fn query_db(
    query: String,
    schema: Schema,
    pool: &IndexerConnectionPool,
) -> ApiResult<axum::Json<Value>> {
    match run_query(query, schema, &pool).await {
        Ok(query_res) => Ok(axum::Json(query_res)),
        Err(e) => {
            error!("query_graph error: {e}");
            Err(e)
        }
    }
}

pub async fn build_schema(
    query: String,
    schema: Schema,
    pool: IndexerConnectionPool,
) -> Result<DynamicSchema, SchemaError> {
    let mut new_query: Object = Object::new("QueryRoot");
    let mut new_schema = DynamicSchema::build(new_query.type_name(), None, None);

    for (type_name, fields) in schema.fields.iter() {
        if type_name != "QueryRoot" {
            let mut obj = Object::new(type_name.to_string());
            for (field_name, field_type) in fields.iter() {
                if field_type.as_str() != "__" {
                    let field_name_clone = field_name.clone();
                    let type_name = field_type.to_string().replace("!", "");
                    let type_type = TypeRef::named_nn(type_name);

                    obj = obj.field(Field::new(
                        field_name.to_string(),
                        type_type,
                        move |ctx: ResolverContext| {
                            let field_name_clone = field_name_clone.clone();
                            FieldFuture::new(async move {
                                let data = ctx.parent_value.try_downcast_ref::<Value>().unwrap().clone();
                                let val = data.get(field_name_clone).unwrap().clone();
                                Ok(Some(DynamicValue::from_json(val).unwrap()))
                            })
                        },
                    ));
                }
            }
            new_schema = new_schema.register(obj);
        } else {
            for (field_name, field_type) in fields.iter() {
                let query_clone = query.clone();
                let schema_clone = schema.clone();
                let pool_clone = pool.clone();

                new_query = new_query.field(Field::new(
                    field_name,
                    TypeRef::named_list_nn(field_type),
                    move |_ctx| {
                        let query_clone = query_clone.clone();
                        let schema_clone = schema_clone.clone();
                        let pool_clone = pool_clone.clone();

                        FieldFuture::new(async move {
                            let result = query_db(query_clone, schema_clone, &pool_clone).await.unwrap();
                            let result = result.get("data").unwrap();
                            let list = result.as_array().unwrap().to_owned();
                            let data = list.into_iter().map(|v| FieldValue::owned_any(v));
                            Ok(Some(FieldValue::list(data)))
                        })
                    },
                ))
            }
        }
    }

    new_schema
        .register(Scalar::new("Address"))
        .register(Scalar::new("AssetId"))
        .register(Scalar::new("Bytes4"))
        .register(Scalar::new("Bytes8"))
        .register(Scalar::new("Bytes32"))
        .register(Scalar::new("Bytes64"))
        .register(Scalar::new("Int4"))
        .register(Scalar::new("Int8"))
        .register(Scalar::new("Int16"))
        .register(Scalar::new("UInt4"))
        .register(Scalar::new("UInt8"))
        .register(Scalar::new("UInt16"))
        .register(Scalar::new("Timestamp"))
        .register(Scalar::new("Color"))
        .register(Scalar::new("ContractId"))
        .register(Scalar::new("Salt"))
        .register(Scalar::new("Json"))
        .register(Scalar::new("MessageId"))
        .register(Scalar::new("Charfield"))
        .register(Scalar::new("Identity"))
        .register(Scalar::new("Blob"))
        .register(new_query)
        .finish()
}