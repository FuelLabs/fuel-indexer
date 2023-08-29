mod test;

use crate::test::prelude::*;
use fuel_indexer_graphql_lib::dynamic::*;
use fuel_indexer_graphql_lib::spec::*;
use std::sync::Arc;
use std::vec;
use tokio::sync::Mutex;

#[tokio::test]
async fn test() {
    let chain_type = DynamicNodeType {
        id: DynamicNodeTypeId::from("Chain"),
        data_fields: vec![DynamicDataField::new("dummy", DynamicDataType::Int)],
        ref_fields: vec![],
        connection_fields: vec![
            DynamicConnectionField {
                id: DynamicFieldId::from("blocks"),
                ref_node_type_id: DynamicNodeTypeId::from("Block"),
            },
            DynamicConnectionField {
                id: DynamicFieldId::from("transactions"),
                ref_node_type_id: DynamicNodeTypeId::from("Transaction"),
            },
        ],
    };
    let block_type = DynamicNodeType {
        id: DynamicNodeTypeId::from("Block"),
        data_fields: vec![
            DynamicDataField::new("number", DynamicDataType::Int),
            DynamicDataField::new("hash", DynamicDataType::String),
            DynamicDataField::new("parent_hash", DynamicDataType::String),
        ],
        ref_fields: vec![
            DynamicRefField {
                id: DynamicFieldId::from("chain"),
                data_field_id: DynamicFieldId::from("chain_id"),
                ref_node_type_id: DynamicNodeTypeId::from("Chain"),
            },
            DynamicRefField {
                id: DynamicFieldId::from("parent"),
                data_field_id: DynamicFieldId::from("parent_hash"),
                ref_node_type_id: DynamicNodeTypeId::from("Block"),
            },
        ],
        connection_fields: vec![
            DynamicConnectionField {
                id: DynamicFieldId::from("blocks"),
                ref_node_type_id: DynamicNodeTypeId::from("Block"),
            },
            DynamicConnectionField {
                id: DynamicFieldId::from("transactions"),
                ref_node_type_id: DynamicNodeTypeId::from("Transaction"),
            },
        ],
    };
    let transaction_type = DynamicNodeType {
        id: DynamicNodeTypeId::from("Transaction"),
        data_fields: vec![
            DynamicDataField::new("index", DynamicDataType::Int),
            DynamicDataField::new("hash", DynamicDataType::String),
            DynamicDataField::new("block_hash", DynamicDataType::String),
        ],
        ref_fields: vec![
            DynamicRefField {
                id: DynamicFieldId::from("chain"),
                data_field_id: DynamicFieldId::from("chain_id"),
                ref_node_type_id: DynamicNodeTypeId::from("Chain"),
            },
            DynamicRefField {
                id: DynamicFieldId::from("block"),
                data_field_id: DynamicFieldId::from("block_hash"),
                ref_node_type_id: DynamicNodeTypeId::from("Block"),
            },
        ],
        connection_fields: vec![],
    };

    let chain: Object = chain_type.clone().into();
    let block: Object = block_type.clone().into();
    let transaction: Object = transaction_type.clone().into();
    let block_connection = <Object as ConnectionObject<DynamicResolver>>::new_connection(
        TypeRef::node("Block"),
    );
    let block_edge = <Object as EdgeObject<DynamicResolver>>::new_edge(
        "Block",
        TypeRef::node("Block"),
    );
    let transaction_connection =
        <Object as ConnectionObject<DynamicResolver>>::new_connection(TypeRef::node(
            "Transaction",
        ));
    let transaction_edge = <Object as EdgeObject<DynamicResolver>>::new_edge(
        "Transaction",
        TypeRef::node("Transaction"),
    );
    let mut schema = Schema::build(TypeRef::QUERY, None, None)
        .register(<Object as PageInfoObject<DynamicResolver>>::new_page_info())
        .register(<Interface as NodeInterface>::new_node())
        .register(chain)
        .register(block)
        .register(transaction)
        .register(block_connection)
        .register(block_edge)
        .register(transaction_connection)
        .register(transaction_edge);

    let nodes = vec![
        DynamicNode::new(
            DynamicNodeTypeId::from("Chain"),
            DynamicNodeLocalId::from("0"),
            json!({
                "id": "0",
                "dummy": "0x00",
            }),
        ),
        DynamicNode::new(
            DynamicNodeTypeId::from("Block"),
            DynamicNodeLocalId::from("0"),
            json!({
                "id": "0",
                "chain_id": "0",
                "number": "0x00",
                "hash": "0x00",
                "parent_hash": "0x00",
            }),
        ),
        DynamicNode::new(
            DynamicNodeTypeId::from("Transaction"),
            DynamicNodeLocalId::from("0"),
            json!({
                "id": "0",
                "chain_id": "0",
                "index": 0,
                "hash": "0x00",
                "block_hash": "0",
            }),
        ),
    ];
    let loader = TestLoader::new(
        nodes,
        vec![
            DynamicEdge::new(
                "ChainHasBlock",
                "0".to_string(),
                "0".to_string(),
                JsonValue::Null,
            ),
            DynamicEdge::new(
                "ChainHasTransaction",
                "0".to_string(),
                "0".to_string(),
                JsonValue::Null,
            ),
        ],
    );
    let resolver = DynamicResolver::new(
        vec![chain_type, block_type, transaction_type],
        Arc::new(Mutex::new(loader)),
    );
    schema = schema.data(resolver);

    let query = <Object as QueryObject<DynamicResolver>>::new_query();

    schema = schema.register(query);

    let schema = schema.finish().unwrap();

    // Execute a query
    let response = execute_query(
        &schema,
        r#"
            query {
                node(id: "Transaction:0") {
                    id
                    ... on Transaction {
                        transactionId: id
                        index
                        hash
                        chain {
                            id
                            blocks(first: 1) {
                                edges {
                                    node {
                                        id
                                    }
                                }
                            }
                        }
                    }
                }
            }
        "#,
        None,
    )
    .await
    .unwrap();

    assert_json_snapshot!(response.data, @r###"
    {
      "node": {
        "id": "Transaction:0",
        "transactionId": "Transaction:0",
        "index": 0,
        "hash": "0x00",
        "chain": {
          "id": "Chain:0",
          "blocks": {
            "edges": [
              {
                "node": {
                  "id": "Block:0"
                }
              }
            ]
          }
        }
      }
    }
    "###
    );
}
