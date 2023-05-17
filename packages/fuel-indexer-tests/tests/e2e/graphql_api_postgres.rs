use actix_service::Service;
use actix_web::test;
use axum::Router;
use fuel_indexer::IndexerService;
use fuel_indexer_lib::config::GraphQLConfig;
use fuel_indexer_lib::manifest::Manifest;
use fuel_indexer_tests::{
    assets, defaults,
    fixtures::{
        api_server_app_postgres, connect_to_deployed_contract, http_client,
        indexer_service_postgres, setup_example_test_fuel_node, test_web::app,
        TestPostgresDb,
    },
    utils::update_test_manifest_asset_paths,
};
use hyper::header::CONTENT_TYPE;
use serde_json::{Number, Value};
use std::collections::HashMap;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

async fn setup_test_components() -> (
    JoinHandle<Result<(), ()>>,
    TestPostgresDb,
    IndexerService,
    Router,
) {
    let node_handle = tokio::spawn(setup_example_test_fuel_node());
    let test_db = TestPostgresDb::new().await.unwrap();
    let srvc = indexer_service_postgres(Some(&test_db.url)).await;
    let (api_app, _rx) = api_server_app_postgres(Some(&test_db.url)).await;

    (node_handle, test_db, srvc, api_app)
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_query_response_with_all_fields_required_postgres() {
    let (node_handle, _test_db, mut srvc, api_app) = setup_test_components().await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(api_app.into_make_service());

    let srv = tokio::spawn(server);
    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/block").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(r#"{ "query": "query { block { id height timestamp }}" }"#)
        .send()
        .await
        .unwrap();

    srv.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert!(data[0]["height"].as_u64().unwrap() > 0);
    assert!(data[0]["timestamp"].as_u64().unwrap() > 0);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_query_response_with_nullable_fields_postgres() {
    let (node_handle, _test_db, mut srvc, api_app) = setup_test_components().await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(api_app.into_make_service());

    let srv = tokio::spawn(server);
    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/optionals").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(r#"{ "query": "query { optionentity { int_required int_optional_some addr_optional_none }}"}"#)
        .send()
        .await
        .unwrap();

    srv.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["int_required"], Value::from(Number::from(100)));
    assert_eq!(data[0]["int_optional_some"], Value::from(Number::from(999)));
    assert_eq!(data[0]["addr_optional_none"], Value::from(None::<&str>));
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_nested_query_response_with_implicit_foreign_keys_postgres() {
    let (node_handle, _test_db, mut srvc, api_app) = setup_test_components().await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(api_app.into_make_service());

    let srv = tokio::spawn(server);
    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/block").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(r#"{ "query": "query { tx { block { id height } id timestamp } }" }"#)
        .send()
        .await
        .unwrap();

    srv.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert!(data[0]["id"].as_i64().is_some());
    assert!(data[0]["id"].as_i64().unwrap() > 0);
    assert!(data[0]["timestamp"].as_i64().is_some());
    assert!(data[0]["timestamp"].as_i64().unwrap() > 0);
    assert!(data[0]["block"]["id"].as_i64().is_some());
    assert!(data[0]["block"]["id"].as_i64().unwrap() > 0);
    assert!(data[0]["block"]["height"].as_i64().is_some());
    assert!(data[0]["block"]["height"].as_i64().unwrap() > 0);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_query_response_with_deeply_nested_query_postgres() {
    let (node_handle, _test_db, mut srvc, api_app) = setup_test_components().await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(api_app.into_make_service());

    let srv = tokio::spawn(server);
    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/deeply_nested").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let deeply_nested_query = HashMap::from([(
        "query",
        "query { 
                bookclub { 
                    id
                    book { 
                        id 
                        name 
                        author { 
                            name 
                            genre { 
                                id 
                                name 
                            } 
                        } 
                        library { 
                            id 
                            name 
                            city { 
                                id 
                                name 
                                region { 
                                    id 
                                    name 
                                    country { 
                                        id 
                                        name 
                                        continent { 
                                            id 
                                            name 
                                            planet { 
                                                id 
                                                name 
                                            } 
                                        } 
                                    } 
                                } 
                            } 
                        } 
                        genre { 
                            id 
                            name 
                        } 
                    } 
                    member { 
                        name 
                        id 
                    } 
                    corporate_sponsor { 
                        id 
                        name 
                        amount 
                        representative { 
                            id 
                            name 
                        } 
                    } 
                } 
            }",
    )]);

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .json(&deeply_nested_query)
        .send()
        .await
        .unwrap();

    srv.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    // Multiple reference to same foreign key table
    assert_eq!(
        data[0]["book"]["author"]["genre"]["name"].as_str(),
        Some("horror")
    );
    assert_eq!(data[0]["book"]["genre"]["name"].as_str(), Some("horror"));

    // Deeply nested foreign keys
    assert_eq!(
        data[0]["book"]["library"]["name"].as_str(),
        Some("Scholar Library")
    );
    assert_eq!(
        data[0]["book"]["library"]["city"]["name"].as_str(),
        Some("Savanna-la-Mar")
    );
    assert_eq!(
        data[0]["book"]["library"]["city"]["region"]["name"].as_str(),
        Some("Westmoreland")
    );
    assert_eq!(
        data[0]["book"]["library"]["city"]["region"]["country"]["name"].as_str(),
        Some("Jamaica")
    );
    assert_eq!(
        data[0]["book"]["library"]["city"]["region"]["country"]["continent"]["name"]
            .as_str(),
        Some("North America")
    );
    assert_eq!(
        data[0]["book"]["library"]["city"]["region"]["country"]["continent"]["planet"]
            ["name"]
            .as_str(),
        Some("Earth")
    );

    // Mix of implicit and explicit foreign keys as well as
    // field name being different from underlying database table
    assert_eq!(
        data[0]["corporate_sponsor"]["name"].as_str(),
        Some("Fuel Labs")
    );
    assert_eq!(data[0]["corporate_sponsor"]["amount"].as_i64(), Some(100));
    assert_eq!(
        data[0]["corporate_sponsor"]["representative"]["name"].as_str(),
        Some("Ava")
    );
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_nested_query_response_with_explicit_foreign_keys_postgres() {
    let (node_handle, _test_db, mut srvc, api_app) = setup_test_components().await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(api_app.into_make_service());

    let srv = tokio::spawn(server);
    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/explicit").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { sportsteam { id name municipality { id name } } }" }"#,
        )
        .send()
        .await
        .unwrap();

    srv.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["name"].as_str(), Some("The Indexers"));
    assert!(data[0]["municipality"]["id"].as_i64().is_some());
    assert!(data[0]["municipality"]["id"].as_i64().unwrap() > 0);
    assert_eq!(
        data[0]["municipality"]["name"].as_str(),
        Some("Republic of Indexia")
    );
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_query_response_with_filter_id_selection_postgres() {
    let (fuel_node_handle, _test_db, mut srvc, api_app) = setup_test_components().await;

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
    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(r#"{ "query": "query { filterentity(id: 1) { id foola maybe_null_bar bazoo } }" }"#)
        .send()
        .await
        .unwrap();

    server_handle.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(1));
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_query_response_with_filter_membership_postgres() {
    let (fuel_node_handle, _test_db, mut srvc, api_app) = setup_test_components().await;

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
    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { filterentity(filter: { foola: { in: [\"beep\", \"boop\"] } } ) { id foola maybe_null_bar bazoo } }" }"#,
        )
        .send()
        .await
        .unwrap();

    server_handle.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(1));
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));
    assert_eq!(data[1]["id"].as_i64(), Some(2));
    assert_eq!(data[1]["foola"].as_str(), Some("boop"));
    assert_eq!(data[1]["maybe_null_bar"].as_i64(), None);
    assert_eq!(data[1]["bazoo"].as_i64(), Some(5));
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_query_response_with_filter_non_null_postgres() {
    let (fuel_node_handle, _test_db, mut srvc, api_app) = setup_test_components().await;

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
    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { filterentity(filter: { has: [maybe_null_bar] } ) { id foola maybe_null_bar bazoo } }" }"#,
        )
        .send()
        .await
        .unwrap();

    server_handle.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(1));
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));
    assert_eq!(data[1]["id"].as_i64(), Some(3));
    assert_eq!(data[1]["foola"].as_str(), Some("blorp"));
    assert_eq!(data[1]["maybe_null_bar"].as_i64(), Some(456));
    assert_eq!(data[1]["bazoo"].as_i64(), Some(1000));
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_query_response_with_filter_complex_comparison_postgres() {
    let (fuel_node_handle, _test_db, mut srvc, api_app) = setup_test_components().await;

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
    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { filterentity(filter: { bazoo: { between: { min: 0, max: 10 } } } ) { id foola maybe_null_bar bazoo } }" }"#,
        )
        .send()
        .await
        .unwrap();

    server_handle.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(1));
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));
    assert_eq!(data[1]["id"].as_i64(), Some(2));
    assert_eq!(data[1]["foola"].as_str(), Some("boop"));
    assert_eq!(data[1]["maybe_null_bar"].as_i64(), None);
    assert_eq!(data[1]["bazoo"].as_i64(), Some(5));
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_query_response_with_filter_simple_comparison_postgres() {
    let (fuel_node_handle, _test_db, mut srvc, api_app) = setup_test_components().await;

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
    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(r#"{ "query": "query { filterentity(filter: { bazoo: { lt: 1000 } } ) { id foola maybe_null_bar bazoo } }" }"#)
        .send()
        .await
        .unwrap();

    server_handle.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(1));
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));
    assert_eq!(data[1]["id"].as_i64(), Some(2));
    assert_eq!(data[1]["foola"].as_str(), Some("boop"));
    assert_eq!(data[1]["maybe_null_bar"].as_i64(), None);
    assert_eq!(data[1]["bazoo"].as_i64(), Some(5));
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_query_response_with_filter_nested_postgres() {
    let (fuel_node_handle, _test_db, mut srvc, api_app) = setup_test_components().await;

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
    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { filterentity(filter: { has: [maybe_null_bar] } ) { id foola maybe_null_bar bazoo inner_entity(filter: { inner_foo: { in: [\"ham\", \"eggs\"] } } ) { id inner_foo inner_bar inner_baz } } }" }"#,
        )
        .send()
        .await
        .unwrap();

    server_handle.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(3));
    assert_eq!(data[0]["foola"].as_str(), Some("blorp"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(456));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1000));
    assert_eq!(data[0]["inner_entity"]["id"].as_i64(), Some(3));
    assert_eq!(data[0]["inner_entity"]["inner_foo"].as_str(), Some("eggs"));
    assert_eq!(data[0]["inner_entity"]["inner_bar"].as_u64(), Some(500));
    assert_eq!(data[0]["inner_entity"]["inner_baz"].as_u64(), Some(600));
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_query_response_with_filter_multiple_on_single_entity_postgres() {
    let (fuel_node_handle, _test_db, mut srvc, api_app) = setup_test_components().await;

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
    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { filterentity(filter: { has: [maybe_null_bar], and: { bazoo: { equals: 1 } } } ) { id foola maybe_null_bar bazoo inner_entity { id inner_foo inner_bar inner_baz } } }" }"#,
        )
        .send()
        .await
        .unwrap();

    server_handle.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(1));
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_query_response_with_filter_negation_postgres() {
    let (fuel_node_handle, _test_db, mut srvc, api_app) = setup_test_components().await;

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
    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{"query": "query { filterentity(filter: { not: { foola: { in: [\"beep\", \"boop\"] } } } ) { id foola maybe_null_bar bazoo } }" }"#,
        )
        .send()
        .await
        .unwrap();

    server_handle.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(3));
    assert_eq!(data[0]["foola"].as_str(), Some("blorp"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(456));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1000));
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_query_response_with_sorted_results_postgres() {
    let (fuel_node_handle, _test_db, mut srvc, api_app) = setup_test_components().await;

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
    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{"query": "query { filterentity(order: { foola: desc }) { id foola } }" }"#,
        )
        .send()
        .await
        .unwrap();

    server_handle.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_i64(), Some(2));
    assert_eq!(data[0]["foola"].as_str(), Some("boop"));
    assert_eq!(data[1]["id"].as_i64(), Some(3));
    assert_eq!(data[1]["foola"].as_str(), Some("blorp"));
    assert_eq!(data[2]["id"].as_i64(), Some(1));
    assert_eq!(data[2]["foola"].as_str(), Some("beep"));
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_query_response_with_alias_and_ascending_offset_and_limited_results_postgres(
) {
    let (fuel_node_handle, _test_db, mut srvc, api_app) = setup_test_components().await;

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
    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let client = http_client();
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{"query": "query { aliased_entities: filterentity(order: { foola: asc }, first: 1, offset: 1) { id foola } }" }"#,
        )
        .send()
        .await
        .unwrap();

    server_handle.abort();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["aliased_entities"][0]["id"].as_i64(), Some(3));
    assert_eq!(
        data[0]["aliased_entities"][0]["foola"].as_str(),
        Some("blorp")
    );
    assert_eq!(data[0]["page_info"]["pages"].as_i64(), Some(3));
}
