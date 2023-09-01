use assert_matches::*;
use async_graphql::dynamic::Schema;
use async_graphql::Request;
use async_graphql_value::ConstValue;
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
                }
                chain(id: "Chain:0") {
                  transactions(first: 1) {
                      nodes {
                          id
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

    assert_matches!(response.data, ConstValue::Object(_));
    if let ConstValue::Object(query_fields) = response.data {
        assert_eq!(query_fields.len(), 2);

        assert_matches!(query_fields.get("transaction"), Some(ConstValue::Object(_)));
        if let Some(ConstValue::Object(transaction_fields)) =
            query_fields.get("transaction")
        {
            assert_eq!(transaction_fields.len(), 1);
            let id_field = transaction_fields.get("id");
            assert_matches!(id_field, Some(ConstValue::String(_)));
            if let Some(ConstValue::String(id)) = id_field {
                assert_eq!(id, "Transaction:0-0-0");
            }
        }

        assert_matches!(query_fields.get("chain"), Some(ConstValue::Object(_)));
        if let Some(ConstValue::Object(chain_fields)) = query_fields.get("chain") {
            assert_eq!(chain_fields.len(), 1);
            let transactions_field = chain_fields.get("transactions");
            assert_matches!(transactions_field, Some(ConstValue::Object(_)));
            if let Some(ConstValue::Object(transactions_fields)) = transactions_field {
                assert_eq!(transactions_fields.len(), 1);
                let nodes_field = transactions_fields.get("nodes");
                assert_matches!(nodes_field, Some(ConstValue::List(_)));
                if let Some(ConstValue::List(nodes)) = nodes_field {
                    assert_eq!(nodes.len(), 1);
                    assert_matches!(nodes.get(0), Some(ConstValue::Object(_)));
                    if let Some(ConstValue::Object(node_fields)) = nodes.get(0) {
                        assert_eq!(node_fields.len(), 1);
                        let id_field = node_fields.get("id");
                        assert_matches!(id_field, Some(ConstValue::String(_)));
                        if let Some(ConstValue::String(id)) = id_field {
                            assert_eq!(id, "Transaction:0-0-0");
                        }
                    }
                }
            }
        }
    }
}
