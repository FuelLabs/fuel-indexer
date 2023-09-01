pub use async_graphql::dynamic::Schema;
pub use async_graphql::Request;
use fuel_indexer_graphql_dyn::testing::prelude::*;
use insta::*;

#[tokio::test]
async fn test() {
    let schema = Schema::build_test();
    let schema = schema.finish().unwrap();

    // Execute a query
    let response = schema
        .execute(Request::new(
            r#"
            query {
                chain(id: "Chain:0") {
                  a: transactions(first: 1) {
                      edges {
                          node {
                              id
                          }
                          cursor
                      }
                      pageInfo {
                          hasNextPage
                          hasPreviousPage
                          startCursor
                          endCursor
                      }
                  }
                  b: transactions(first: 1) {
                      edges {
                          node {
                              id
                          }
                          cursor
                      }
                      pageInfo {
                          hasNextPage
                          hasPreviousPage
                          startCursor
                          endCursor
                      }
                  }
                }
            }
        "#,
        ))
        .await
        .into_result()
        .unwrap();

    assert_json_snapshot!(response.data);
}
