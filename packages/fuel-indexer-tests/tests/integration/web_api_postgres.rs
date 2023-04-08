use fuel_indexer_lib::config::GraphQLConfig;
use fuel_indexer_postgres as postgres;
use fuel_indexer_tests::assets::{
    SIMPLE_WASM_MANIFEST, SIMPLE_WASM_SCHEMA, SIMPLE_WASM_WASM,
};
use fuel_indexer_tests::fixtures::{
    api_server_app_postgres, authenticated_api_server_app_postgres, http_client,
    indexer_service_postgres, TestPostgresDb,
};
use hyper::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const SIGNATURE: &str = "cb19384361af5dd7fec2a0052ca49d289f997238ea90590baf47f16ff0a33fb20170a43bd20208ce16daf443bad06dd66c1d1bf73f48b5ae53de682a5731d7d9";
const NONCE: &str = "ea35be0c98764e7ca06d02067982e3b4";

#[tokio::test]
#[cfg(all(feature = "postgres"))]
async fn test_metrics_endpoint_returns_proper_count_of_metrics_postgres() {
    let test_db = TestPostgresDb::new().await.unwrap();
    let _srvc = indexer_service_postgres(Some(&test_db.url)).await;
    let app = api_server_app_postgres(Some(&test_db.url)).await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(app.into_make_service());

    let srv = tokio::spawn(server);

    let client = http_client();
    let _ = client
        .get("http://localhost:29987/api/health")
        .send()
        .await
        .unwrap();

    let resp = client
        .get("http://localhost:29987/api/metrics")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    srv.abort();
    assert_eq!(resp.split('\n').count(), 112);
}

#[tokio::test]
#[cfg(all(feature = "postgres"))]
async fn test_database_postgres_metrics_properly_increments_counts_when_queries_are_made()
{
    let test_db = TestPostgresDb::new().await.unwrap();
    let _ = indexer_service_postgres(Some(&test_db.url)).await;
    let app = api_server_app_postgres(Some(&test_db.url)).await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(app.into_make_service());

    let _srv = tokio::spawn(server);

    let mut conn = test_db.pool.acquire().await.unwrap();
    let _ = postgres::execute_query(&mut conn, "SELECT 1;".into()).await;
    let _ = postgres::execute_query(&mut conn, "SELECT 1;".into()).await;

    let client = http_client();
    let _ = client
        .get("http://localhost:29987/api/health")
        .send()
        .await
        .unwrap();

    let resp = client
        .get("http://localhost:29987/api/metrics")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let categories = resp.split('\n').collect::<Vec<&str>>();

    assert_eq!(
        categories[18],
        "# HELP postgres_execute_query_calls Count of calls to postgres execute_query_calls."
    );
    assert_eq!(
        categories[19],
        "# TYPE postgres_execute_query_calls counter"
    );

    assert!(
        categories[20].split(' ').collect::<Vec<&str>>()[1]
            .to_string()
            .parse::<i64>()
            .unwrap()
            >= 1,
    );
}

#[tokio::test]
#[cfg(all(feature = "postgres"))]
async fn test_asset_upload_endpoint_properly_adds_assets_to_database_postgres() {
    let test_db = TestPostgresDb::new().await.unwrap();
    let app = api_server_app_postgres(Some(&test_db.url)).await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(app.into_make_service());

    let srv = tokio::spawn(server);

    let mut conn = test_db.pool.acquire().await.unwrap();
    let is_index_registered = postgres::index_is_registered(
        &mut conn,
        "test_namespace",
        "simple_wasm_executor",
    )
    .await
    .unwrap();
    assert!(is_index_registered.is_none());

    let manifest_file =
        multipart::Part::stream(SIMPLE_WASM_MANIFEST).file_name("simple_wasm.yaml");
    let schema_file =
        multipart::Part::stream(SIMPLE_WASM_SCHEMA).file_name("simple_wasm.graphql");
    let wasm_file =
        multipart::Part::stream(SIMPLE_WASM_WASM).file_name("simple_wasm.wasm");

    let form = multipart::Form::new()
        .part("manifest", manifest_file)
        .part("schema", schema_file)
        .part("wasm", wasm_file);

    let client = http_client();
    let resp = client
        .post("http://localhost:29987/api/index/test_namespace/simple_wasm_executor")
        .multipart(form)
        .header(CONTENT_TYPE, "multipart/form-data".to_owned())
        .header(AUTHORIZATION, "foo".to_owned())
        .send()
        .await
        .unwrap();

    srv.abort();

    assert!(resp.status().is_success());

    let is_index_registered = postgres::index_is_registered(
        &mut conn,
        "test_namespace",
        "simple_wasm_executor",
    )
    .await
    .unwrap();

    assert!(is_index_registered.is_some());
}

#[derive(Serialize, Debug)]
struct SignatureRequest {
    signature: String,
    message: String,
}

#[derive(Deserialize, Debug)]
struct SignatureResponse {
    token: Option<String>,
}

#[tokio::test]
#[cfg(all(feature = "postgres"))]
async fn test_signature_route_validates_signature_expires_nonce_and_creates_jwt() {
    let test_db = TestPostgresDb::new().await.unwrap();
    let app = authenticated_api_server_app_postgres(Some(&test_db.url)).await;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let expiry = now + 3600;

    let mut conn = test_db.pool.acquire().await.unwrap();
    let _ = sqlx::QueryBuilder::new("INSERT INTO nonce (uid, expiry) VALUES ($1, $2)")
        .build()
        .bind(NONCE)
        .bind(expiry as i64)
        .execute(&mut conn)
        .await
        .unwrap();

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(app.into_make_service());

    let _srv = tokio::spawn(server);

    let resp = http_client()
        .post("http://localhost:29987/api/auth/signature")
        .header(CONTENT_TYPE, "application/json".to_owned())
        .json(&SignatureRequest {
            signature: SIGNATURE.to_string(),
            message: NONCE.to_string(),
        })
        .send()
        .await
        .unwrap();

    let res: SignatureResponse = resp.json().await.unwrap();

    assert!(res.token.is_some());
    assert!(res.token.unwrap().len() > 300);

    let _ = sqlx::QueryBuilder::new("DELETE FROM nonce WHERE uid = $1")
        .build()
        .bind(NONCE)
        .execute(&mut conn)
        .await
        .unwrap();
}
