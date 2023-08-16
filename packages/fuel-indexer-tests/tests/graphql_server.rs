use fuel_indexer_tests::fixtures::{
    mock_request, setup_web_test_components, WebTestComponents,
};
use fuel_indexer_utils::uid;
use hyper::header::CONTENT_TYPE;
use serde_json::{Number, Value};
use std::collections::HashMap;

#[actix_web::test]
async fn test_entity_with_required_and_optional_fields() {
    let WebTestComponents {
        server,
        db: _db,
        client,
        ..
    } = setup_web_test_components(None).await;

    mock_request("/block").await;

    // All required
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(r#"{ "query": "query { blockentity { id height timestamp }}" }"#)
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["height"].as_u64().unwrap(), 1);
    assert!(data[0]["timestamp"].as_u64().unwrap() > 0);

    assert!(data[1]["height"].as_u64().unwrap() > 0);
    assert!(data[1]["timestamp"].as_u64().unwrap() > 0);

    // Optionals
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(r#"{ "query": "query { optionentity { int_required int_optional_some addr_optional_none }}"}"#)
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["int_required"], Value::from(Number::from(100)));
    assert_eq!(data[0]["int_optional_some"], Value::from(Number::from(999)));
    assert_eq!(data[0]["addr_optional_none"], Value::from(None::<&str>));

    server.abort();
}

#[actix_web::test]
async fn test_entity_with_foreign_keys() {
    let WebTestComponents {
        server,
        db: _db,
        client,
        ..
    } = setup_web_test_components(None).await;

    mock_request("/block").await;

    // Implicit foreign keys
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(r#"{ "query": "query { txentity { block { id height } id timestamp } }" }"#)
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert!(data[0]["timestamp"].as_i64().is_some());
    assert!(data[0]["timestamp"].as_i64().unwrap() > 0);
    assert!(data[0]["block"]["height"].as_i64().is_some());
    assert!(data[0]["block"]["height"].as_i64().unwrap() > 0);

    // Explicit foreign keys
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { sportsteam { id name municipality { id name } } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["name"].as_str(), Some("The Indexers"));
    assert!(data[0]["muipality"]["id"].as_str().is_some());
    assert_eq!(
        data[0]["municipality"]["name"].as_str(),
        Some("Republic of Indexia")
    );

    server.abort();
}

#[actix_web::test]
async fn test_deeply_nested_entity() {
    let WebTestComponents {
        server,
        db: _db,
        client,
        ..
    } = setup_web_test_components(None).await;

    mock_request("/deeply_nested").await;

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

    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .json(&deeply_nested_query)
        .send()
        .await
        .unwrap();

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

    server.abort();
}

#[actix_web::test]
async fn test_filtering() {
    let WebTestComponents {
        server,
        db: _db,
        client,
        ..
    } = setup_web_test_components(None).await;

    mock_request("/ping").await;

    let id = uid([1]).to_string();

    // ID selection
    let _resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(format!(r#"{{ "query": "query {{ filterentity(id: "{id}" ) {{ id foola maybe_null_bar bazoo }} }}" }}"#))
        .send()
        .await
        .unwrap();

    // let body = resp.text().await.unwrap();
    // println!(">> DATA: {body:?}");
    // let v: Value = serde_json::from_str(&body).unwrap();
    // let data = v["data"].as_array().expect("data is not an array");

    // assert!(data[0]["id"].as_str().is_some());
    // assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    // assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    // assert_eq!(data[0]["bazoo"].as_i64(), Some(1));

    // Set membership
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { filterentity(filter: { foola: { in: [\"beep\", \"boop\"] } } ) { id foola maybe_null_bar bazoo } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert!(data[0]["id"].as_str().is_some());
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));
    assert!(data[1]["id"].as_str().is_some());
    assert_eq!(data[1]["foola"].as_str(), Some("boop"));
    assert_eq!(data[1]["maybe_null_bar"].as_i64(), None);
    assert_eq!(data[1]["bazoo"].as_i64(), Some(5));

    // Non-null
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { filterentity(filter: { has: [maybe_null_bar] } ) { id foola maybe_null_bar bazoo } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert!(data[0]["id"].as_str().is_some());
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));
    assert!(data[1]["id"].as_str().is_some());
    assert_eq!(data[1]["foola"].as_str(), Some("blorp"));
    assert_eq!(data[1]["maybe_null_bar"].as_i64(), Some(456));
    assert_eq!(data[1]["bazoo"].as_i64(), Some(1000));

    // Complex comparison
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { filterentity(filter: { bazoo: { between: { min: 0, max: 10 } } } ) { id foola maybe_null_bar bazoo } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert!(data[0]["id"].as_str().is_some());
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));
    assert!(data[1]["id"].as_str().is_some());
    assert_eq!(data[1]["foola"].as_str(), Some("boop"));
    assert_eq!(data[1]["maybe_null_bar"].as_i64(), None);
    assert_eq!(data[1]["bazoo"].as_i64(), Some(5));

    // Simple comparison
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(r#"{ "query": "query { filterentity(filter: { bazoo: { lt: 1000 } } ) { id foola maybe_null_bar bazoo } }" }"#)
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert!(data[0]["id"].as_str().is_some());
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));
    assert!(data[1]["id"].as_str().is_some());
    assert_eq!(data[1]["foola"].as_str(), Some("boop"));
    assert_eq!(data[1]["maybe_null_bar"].as_i64(), None);
    assert_eq!(data[1]["bazoo"].as_i64(), Some(5));

    // Nested filters
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { filterentity(filter: { has: [maybe_null_bar] } ) { id foola maybe_null_bar bazoo inner_entity(filter: { inner_foo: { in: [\"ham\", \"eggs\"] } } ) { id inner_foo inner_bar inner_baz } } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert!(data[0]["id"].as_str().is_some());
    assert_eq!(data[0]["foola"].as_str(), Some("blorp"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(456));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1000));
    assert!(data[0]["inner_entity"]["id"].as_str().is_some());
    assert_eq!(data[0]["inner_entity"]["inner_foo"].as_str(), Some("eggs"));
    assert_eq!(data[0]["inner_entity"]["inner_bar"].as_u64(), Some(500));
    assert_eq!(data[0]["inner_entity"]["inner_baz"].as_u64(), Some(600));

    // Multiple filters on single entity
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{ "query": "query { filterentity(filter: { has: [maybe_null_bar], and: { bazoo: { equals: 1 } } } ) { id foola maybe_null_bar bazoo inner_entity { id inner_foo inner_bar inner_baz } } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert!(data[0]["id"].as_str().is_some());
    assert_eq!(data[0]["foola"].as_str(), Some("beep"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(123));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1));

    // Negation
    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{"query": "query { filterentity(filter: { not: { foola: { in: [\"beep\", \"boop\"] } } } ) { id foola maybe_null_bar bazoo } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert!(data[0]["id"].as_str().is_some());
    assert_eq!(data[0]["foola"].as_str(), Some("blorp"));
    assert_eq!(data[0]["maybe_null_bar"].as_i64(), Some(456));
    assert_eq!(data[0]["bazoo"].as_i64(), Some(1000));

    server.abort();
}

#[actix_web::test]
async fn test_sorting() {
    let WebTestComponents {
        server,
        db: _db,
        client,
        ..
    } = setup_web_test_components(None).await;

    mock_request("/ping").await;

    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{"query": "query { filterentity(order: { foola: desc }) { id foola } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(data[0]["id"].as_str().unwrap(), uid([2]).to_string());
    assert_eq!(data[0]["foola"].as_str(), Some("boop"));
    assert_eq!(data[1]["id"].as_str().unwrap(), uid([3]).to_string());
    assert_eq!(data[1]["foola"].as_str(), Some("blorp"));
    assert_eq!(data[2]["id"].as_str().unwrap(), uid([1]).to_string());
    assert_eq!(data[2]["foola"].as_str(), Some("beep"));

    server.abort();
}

#[actix_web::test]
async fn test_aliasing_and_pagination() {
    let WebTestComponents {
        server,
        db: _db,
        client,
        ..
    } = setup_web_test_components(None).await;

    mock_request("/ping").await;

    let resp = client
        .post("http://127.0.0.1:29987/api/graph/fuel_indexer_test/index1")
        .header(CONTENT_TYPE, "application/graphql".to_owned())
        .body(
            r#"{"query": "query { aliased_entities: filterentity(order: { foola: asc }, first: 1, offset: 1) { id foola } }" }"#,
        )
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let data = v["data"].as_array().expect("data is not an array");

    assert_eq!(
        data[0]["aliased_entities"][0]["foola"].as_str(),
        Some("blorp")
    );
    assert_eq!(data[0]["page_info"]["pages"].as_i64(), Some(3));

    server.abort();
}
