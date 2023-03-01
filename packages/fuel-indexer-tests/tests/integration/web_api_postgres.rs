use fuel_indexer_api_server::api::GraphQlApi;
use fuel_indexer_lib::config::GraphQLConfig;
use fuel_indexer_postgres as postgres;
use fuel_indexer_tests::assets::{
    SIMPLE_WASM_MANIFEST, SIMPLE_WASM_SCHEMA, SIMPLE_WASM_WASM,
};
use fuel_indexer_tests::fixtures::{
    api_server_app_postgres, http_client, indexer_service_postgres,
    postgres_connection_pool,
};
use hyper::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::{multipart, Body};
use tokio::task::spawn;

#[tokio::test]
#[cfg(all(feature = "postgres"))]
async fn test_metrics_endpoint_returns_proper_count_of_metrics_postgres() {
    let _ = indexer_service_postgres().await;
    let app = api_server_app_postgres().await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(app.into_make_service());

    let server_handle = tokio::spawn(server);

    let client = http_client();
    let _ = client
        .get("http://127.0.0.1:29987/api/health")
        .send()
        .await
        .unwrap();

    let resp = client
        .get("http://127.0.0.1:29987/api/metrics")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    server_handle.abort();

    assert_eq!(resp.split('\n').count(), 106);
}

#[tokio::test]
#[cfg(all(feature = "postgres"))]
async fn test_database_postgres_metrics_properly_increments_counts_when_queries_are_made()
{
    let _ = indexer_service_postgres().await;
    let app = api_server_app_postgres().await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(app.into_make_service());

    let server_handle = tokio::spawn(server);

    let pool = postgres_connection_pool().await;
    let mut conn = pool.acquire().await.unwrap();
    let _ = postgres::execute_query(&mut conn, "SELECT 1;".into()).await;
    let _ = postgres::execute_query(&mut conn, "SELECT 1;".into()).await;

    let client = http_client();
    let _ = client
        .get("http://127.0.0.1:29987/api/health")
        .send()
        .await
        .unwrap();

    let resp = client
        .get("http://127.0.0.1:29987/api/metrics")
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
    let pool = postgres_connection_pool().await;
    let mut conn = pool.acquire().await.unwrap();

    let app = api_server_app_postgres().await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(app.into_make_service());

    let server_handle = tokio::spawn(server);

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
        .post("http://127.0.0.1:29987/api/index/test_namespace/simple_wasm_executor")
        .multipart(form)
        .header(CONTENT_TYPE, "multipart/form-data".to_owned())
        .header(AUTHORIZATION, "foo".to_owned())
        .send()
        .await
        .unwrap();

    server_handle.abort();

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
