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
                transaction(id: "Transaction:0-0-0") {
                    id
                    index
                    hash
                    gasPrice
                    gasLimit
                    blockHash
                }
            }
        "#,
        ))
        .await
        .into_result()
        .unwrap();

    assert_json_snapshot!(response.data, @r###"
    {
      "transaction": {
        "id": "Transaction:0-0-0",
        "index": 0,
        "hash": "0x00",
        "gasPrice": 12312,
        "gasLimit": 32423,
        "blockHash": "0"
      }
    }
    "###
    );
}
