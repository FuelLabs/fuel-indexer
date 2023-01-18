use actix_service::Service;
use actix_web::test;
use fuel_indexer::IndexerService;
use fuel_indexer_lib::config::GraphQLConfig;
use fuel_indexer_lib::manifest::Manifest;
use fuel_indexer_tests::{
    assets, defaults,
    fixtures::{
        api_server_app_sqlite, connect_to_deployed_contract, http_client,
        indexer_service_sqlite, test_web::app,
    },
    utils::update_test_manifest_asset_paths,
};
use hyper::header::{AUTHORIZATION, CONTENT_TYPE};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use tokio::task::spawn;
use tokio::time::{sleep, Duration};

#[derive(Deserialize, Serialize)]
struct NestedQueryResponse {
    id: String,
    input_data: String,
    timestamp: u64,
    block: Block,
}

#[derive(Deserialize, Serialize)]
struct Block {
    id: String,
    height: u64,
    timestamp: u64,
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "sqlite"))]
async fn test_can_return_query_response_with_all_fields_required_sqlite() {
    let mut srvc = indexer_service_sqlite().await;
    let api_app = api_server_app_sqlite().await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(api_app.into_make_service());

    let server_handle = tokio::spawn(server);
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/block").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test")
        .header(CONTENT_TYPE, "application/json".to_owned())
        .body(r#"{"query": "query { block { id height timestamp }}", "params": "b"}"#)
        .send()
        .await
        .unwrap();

    server_handle.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();

    assert!(v[0]["height"].as_u64().unwrap() > 0);
    assert!(v[0]["timestamp"].as_u64().unwrap() > 0);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "sqlite"))]
async fn test_can_return_query_response_with_nullable_fields_sqlite() {
    let mut srvc = indexer_service_sqlite().await;
    let api_app = api_server_app_sqlite().await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(api_app.into_make_service());

    let server_handle = tokio::spawn(server);
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/optionals").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test")
        .header(CONTENT_TYPE, "application/json".to_owned())
        .body(r#"{"query": "query { optionentity { id int_required int_optional_some addr_optional_none }}", "params": "b"}"#)
        .send()
        .await
        .unwrap();

    server_handle.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(v[0]["id"], Value::from(Number::from(1)));
    assert_eq!(v[0]["int_required"], Value::from(Number::from(100)));
    assert_eq!(v[0]["int_optional_some"], Value::from(Number::from(999)));
    assert_eq!(v[0]["addr_optional_none"], Value::from(None::<&str>));
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "sqlite"))]
async fn test_can_return_query_response_with_nested_implicit_foreign_key_fields_sqlite() {
    let mut srvc = indexer_service_sqlite().await;
    let api_app = api_server_app_sqlite().await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(api_app.into_make_service());

    let server_handle = tokio::spawn(server);
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/block").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test")
        .header(CONTENT_TYPE, "application/json".to_owned())
        .body(r#"{"query": "query { tx { id input_data timestamp block { id height timestamp }}}", "params": "b"}"#)
        .send()
        .await
        .unwrap();

    server_handle.abort();

    let body = resp.text().await.unwrap();
    let nested_responses: Vec<NestedQueryResponse> = serde_json::from_str(&body).unwrap();

    assert!(!nested_responses[0].id.is_empty());
    assert!(!nested_responses[0].input_data.is_empty());
    assert!(nested_responses[0].timestamp > 0);

    assert!(!nested_responses[0].block.id.is_empty());
    assert!(nested_responses[0].block.height > 0);
    assert!(nested_responses[0].block.timestamp > 0);
}
