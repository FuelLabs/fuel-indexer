use fuel_indexer::IndexerConfig;
use fuel_indexer_lib::config::{
    auth::AuthenticationStrategy, defaults as config_defaults, AuthenticationConfig,
};
use fuel_indexer_postgres as postgres;
use fuel_indexer_tests::{
    assets,
    fixtures::{http_client, setup_web_test_components, WebTestComponents},
};
use hyper::header::CONTENT_TYPE;
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

const SIGNATURE: &str = "cb19384361af5dd7fec2a0052ca49d289f997238ea90590baf47f16ff0a33fb20170a43bd20208ce16daf443bad06dd66c1d1bf73f48b5ae53de682a5731d7d9";
const NONCE: &str = "ea35be0c98764e7ca06d02067982e3b4";

#[tokio::test]
async fn test_metrics_endpoint_returns_proper_count_of_metrics_postgres() {
    let WebTestComponents { server, .. } = setup_web_test_components(None).await;

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

    server.abort();
    assert!((resp.split('\n').count() >= 127));
}

#[tokio::test]
async fn test_database_postgres_metrics_properly_increments_counts_when_queries_are_made()
{
    let WebTestComponents { server, db, .. } = setup_web_test_components(None).await;

    let mut conn = db.pool.acquire().await.unwrap();
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

    assert!(
        categories[18].split(' ').collect::<Vec<&str>>()[1]
            .to_string()
            .parse::<i64>()
            .unwrap()
            >= 0,
    );

    server.abort();
}

#[tokio::test]
async fn test_asset_upload_endpoint_properly_adds_assets_to_database_postgres() {
    let WebTestComponents {
        server,
        db,
        rx: _rx,
        ..
    } = setup_web_test_components(None).await;

    let mut conn = db.pool.acquire().await.unwrap();
    let is_indexer_registered =
        postgres::get_indexer(&mut conn, "test_namespace", "simple_wasm_executor")
            .await
            .unwrap();
    assert!(is_indexer_registered.is_none());

    let manifest_file = multipart::Part::stream(assets::SIMPLE_WASM_MANIFEST)
        .file_name("simple_wasm.yaml");
    let schema_file = multipart::Part::stream(assets::SIMPLE_WASM_SCHEMA)
        .file_name("simple_wasm.graphql");
    let wasm_file =
        multipart::Part::stream(assets::SIMPLE_WASM_WASM).file_name("simple_wasm.wasm");

    let form = multipart::Form::new()
        .part("manifest", manifest_file)
        .part("schema", schema_file)
        .part("wasm", wasm_file);

    let client = http_client();
    let resp = client
        .post("http://localhost:29987/api/index/test_namespace/simple_wasm_executor")
        .multipart(form)
        .header(CONTENT_TYPE, "multipart/form-data".to_owned())
        .send()
        .await
        .unwrap();

    server.abort();

    assert!(resp.status().is_success());

    let is_indexer_registered =
        postgres::get_indexer(&mut conn, "test_namespace", "simple_wasm_executor")
            .await
            .unwrap();

    assert!(is_indexer_registered.is_some());
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
async fn test_signature_route_validates_signature_expires_nonce_and_creates_jwt() {
    let config = IndexerConfig {
        authentication: AuthenticationConfig{
            enabled: true,
            strategy: Some(AuthenticationStrategy::JWT),
            jwt_secret: Some("6906573247652854078288872150120717701634680141358560585446649749925714230966".to_string()),
            jwt_issuer: Some("FuelLabs".to_string()),
            jwt_expiry: Some(config_defaults::JWT_EXPIRY_SECS)
        },
        ..IndexerConfig::default()
    };

    let WebTestComponents { server, db, .. } =
        setup_web_test_components(Some(config)).await;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let expiry = now + 3600;

    let mut conn = db.pool.acquire().await.unwrap();
    let _ = sqlx::QueryBuilder::new("INSERT INTO nonce (uid, expiry) VALUES ($1, $2)")
        .build()
        .bind(NONCE)
        .bind(expiry as i64)
        .execute(&mut conn)
        .await
        .unwrap();

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

    server.abort();
}

#[actix_web::test]
async fn test_querying_sql_endpoint_when_sql_not_enabled_returns_404() {
    let WebTestComponents { server, .. } = setup_web_test_components(None).await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/sql/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/json".to_owned())
        .body(r#"{ "query": "SELECT COUNT(*)" }"#)
        .send()
        .await
        .unwrap();

    server.abort();

    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_querying_sql_endpoint_when_sql_is_enabled_returns_actual_query_response() {
    let config = IndexerConfig {
        accept_sql_queries: true,
        ..IndexerConfig::default()
    };

    let WebTestComponents { server, db, .. } =
        setup_web_test_components(Some(config)).await;

    let mut conn = db.pool.acquire().await.unwrap();

    let _ = sqlx::QueryBuilder::new("INSERT INTO fuel_indexer_test_index1.pingentity  (id, value, message, object) VALUES ($1, $2, $3, $4::bytea)")
        .build()
        .bind(123456789)
        .bind(987654321)
        .bind("My message")
        .bind("fake object")
        .execute(&mut conn)
        .await
        .unwrap();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/sql/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/json".to_owned())
        .body(
            r#"{ "query": "SELECT json_agg(t) FROM (SELECT COUNT(*) FROM fuel_indexer_test_index1.pingentity) t" }"#,
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();

    let v: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["data"][0][0]["count"], 1);

    server.abort();
}

#[actix_web::test]
async fn test_querying_sql_endpoint_when_sql_is_enabled_returns_error_for_non_supported_queries(
) {
    let config = IndexerConfig {
        accept_sql_queries: true,
        ..IndexerConfig::default()
    };

    let WebTestComponents { server, db, .. } =
        setup_web_test_components(Some(config)).await;

    let mut conn = db.pool.acquire().await.unwrap();

    let _ = sqlx::QueryBuilder::new("INSERT INTO fuel_indexer_test_index1.pingentity  (id, value, message, object) VALUES ($1, $2, $3, $4::bytea)")
        .build()
        .bind(123456789)
        .bind(987654321)
        .bind("My message")
        .bind("fake object")
        .execute(&mut conn)
        .await
        .unwrap();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/sql/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/json".to_owned())
        .body(r#"{ "query": "DROP SCHEMA fuel_indexer_test" }"#)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body = resp.text().await.unwrap();

    let v: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["success"], "false");
    assert_eq!(v["details"], "Error: Operation is not supported.");

    server.abort();
}
