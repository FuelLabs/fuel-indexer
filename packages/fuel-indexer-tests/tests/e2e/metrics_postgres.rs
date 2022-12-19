use fuel_indexer_postgres as postgres;
use fuel_indexer_tests::fixtures::{http_client, postgres_connection_pool};

#[tokio::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_metrics_endpoint_returns_proper_count_of_metrics_postgres() {
    let client = http_client();
    let resp = client
        .get("http://127.0.0.1:29987/api/metrics")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(resp.split('\n').count(), 190);
}

#[tokio::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_database_postgres_metrics_properly_increments_counts_when_queries_are_made()
{
    let pool = postgres_connection_pool().await;
    let mut conn = pool.acquire().await.unwrap();
    let _ = postgres::execute_query(&mut conn, "SELECT 1;".into());
    let _ = postgres::execute_query(&mut conn, "SELECT 1;".into());

    let client = http_client();

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
