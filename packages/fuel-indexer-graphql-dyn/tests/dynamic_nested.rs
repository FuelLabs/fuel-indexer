use async_graphql::dynamic::Schema;
use async_graphql::Request;
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
                node(id: "Transaction:0-0-0") {
                    id
                    ... on Transaction {
                        transactionId: id
                        index
                        hash
                        chain {
                            id
                            blocks(first: 1) {
                                totalCount
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
                }
            }
        "#,
        ))
        .await
        .into_result()
        .unwrap();

    assert_json_snapshot!(response.data, @r###"
    {
      "node": {
        "id": "Transaction:0-0-0",
        "transactionId": "Transaction:0-0-0",
        "index": 0,
        "hash": "0x00",
        "chain": {
          "id": "Chain:0",
          "blocks": {
            "totalCount": 1,
            "edges": [
              {
                "node": {
                  "id": "Block:0-0"
                },
                "cursor": "0"
              }
            ],
            "pageInfo": {
              "hasNextPage": false,
              "hasPreviousPage": false,
              "startCursor": "0",
              "endCursor": "0"
            }
          }
        }
      }
    }
    "###
    );
}
