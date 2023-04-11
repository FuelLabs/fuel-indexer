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

async fn setup_test_components(
    number_of_contracts: u8,
) -> (
    JoinHandle<Result<(), ()>>,
    TestPostgresDb,
    IndexerService,
    Router,
) {
    let node_handle = tokio::spawn(setup_example_test_fuel_node(number_of_contracts));
    let _test_db = TestPostgresDb::new().await.unwrap();
    let srvc = indexer_service_postgres(Some(&_test_db.url)).await;
    let api_app = api_server_app_postgres(Some(&_test_db.url)).await;

    (node_handle, _test_db, srvc, api_app)
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_return_query_response_with_all_fields_required_postgres() {
    let (node_handle, _test_db, mut srvc, api_app) = setup_test_components(1).await;

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
        .post("http://localhost:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/json".to_owned())
        .body(r#"{"query": "query { block { id height timestamp }}", "params": "b"}"#)
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
    let (node_handle, _test_db, mut srvc, api_app) = setup_test_components(1).await;

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
        .post("http://localhost:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/json".to_owned())
        .body(r#"{"query": "query { optionentity { int_required int_optional_some addr_optional_none }}", "params": "b"}"#)
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
    let (node_handle, _test_db, mut srvc, api_app) = setup_test_components(1).await;

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
        .post("http://localhost:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/json".to_owned())
        .body(
            r#"{"query": "query { tx { block { id height } id timestamp }}", "params": "b"}"#,
        )
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
    let (node_handle, _test_db, mut srvc, api_app) = setup_test_components(1).await;

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

    let deeply_nested_query = HashMap::from([
        (
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
        ),
        ("params", "b"),
    ]);

    let client = http_client();
    let resp = client
        .post("http://localhost:29987/api/graph/fuel_indexer_test/index1")
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
    let (node_handle, _test_db, mut srvc, api_app) = setup_test_components(1).await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(api_app.into_make_service());

    let srv = tokio::spawn(server);
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);
    println!("successfully updated manifest asset paths");

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/explicit").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let client = http_client();
    let resp = client
        .post("http://localhost:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/json".to_owned())
        .body(
            r#"{"query": "query { sportsteam { id name municipality { id name } } }", "params": "b"}"#,
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
