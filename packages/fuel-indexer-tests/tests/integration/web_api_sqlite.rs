use fuel_indexer_api_server::api::GraphQlApi;
use fuel_indexer_lib::config::GraphQLConfig;
use fuel_indexer_sqlite as sqlite;
use fuel_indexer_tests::fixtures::{
    api_server_app_sqlite, http_client, indexer_service_sqlite, sqlite_connection_pool,
};
use tokio::task::spawn;

#[tokio::test]
#[cfg(all(feature = "sqlite"))]
async fn test_metrics_endpoint_returns_proper_count_of_metrics_sqlite() {
    let _ = indexer_service_sqlite().await;
    let app = api_server_app_sqlite().await;

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

    assert_eq!(resp.split('\n').count(), 190);
}

#[tokio::test]
#[cfg(all(feature = "sqlite"))]
async fn test_database_sqlite_metrics_properly_increments_counts_when_queries_are_made() {
    let _ = indexer_service_sqlite().await;
    let app = api_server_app_sqlite().await;

    let server = axum::Server::bind(&GraphQLConfig::default().into())
        .serve(app.into_make_service());

    let server_handle = tokio::spawn(server);
    let pool = sqlite_connection_pool().await;
    let mut conn = pool.acquire().await.unwrap();
    let _ = sqlite::execute_query(&mut conn, "SELECT 1;".into());
    let _ = sqlite::execute_query(&mut conn, "SELECT 1;".into());

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
        categories[108],
        "# HELP sqlite_execute_query_calls Count of calls to sqlite execute_query_calls."
    );
    assert_eq!(categories[109], "# TYPE sqlite_execute_query_calls counter");

    assert!(
        categories[110].split(' ').collect::<Vec<&str>>()[1]
            .to_string()
            .parse::<i64>()
            .unwrap()
            >= 1,
    );
}
