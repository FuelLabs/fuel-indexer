mod test;

use crate::test::prelude::*;
use fuel_indexer_graphql_lib::json_resolver::*;

fn build_schema() -> SchemaBuilder {
    let mut schema = Schema::build(TypeRef::QUERY, None, None)
        .register(<Object as PageInfoObject<JsonResolver>>::new_page_info())
        .register(<Interface as NodeInterface>::new_node());
    let query = <Object as QueryObject<JsonResolver>>::new_query();

    // Some nodes
    let chain = <Object as NodeObject<JsonResolver>>::new_node("Chain");
    let mut block = NodeObject::<JsonResolver>::new_node("Block");
    block = <Object as NodeObject<JsonResolver>>::data_field(
        block,
        "number",
        TypeRef::named_nn(TypeRef::INT),
    );
    block = <Object as NodeObject<JsonResolver>>::data_field(
        block,
        "hash",
        TypeRef::named_nn(TypeRef::STRING),
    );
    block = <Object as NodeObject<JsonResolver>>::ref_field(
        block,
        "chain",
        TypeRef::named_nn(TypeRef::node("Chain")),
    );
    let mut transaction = <Object as NodeObject<JsonResolver>>::new_node("Transaction");
    transaction = <Object as NodeObject<JsonResolver>>::data_field(
        transaction,
        "index",
        TypeRef::named_nn(TypeRef::INT),
    );
    transaction = <Object as NodeObject<JsonResolver>>::data_field(
        transaction,
        "hash",
        TypeRef::named_nn(TypeRef::STRING),
    );

    schema = schema
        .register(chain)
        .register(block)
        .register(transaction)
        .register(query);

    schema
}

#[tokio::test]
async fn resolve() {
    // Build the schema
    let mut schema = build_schema();
    let resolver = JsonResolver::new(vec![
        Node(
            "Chain:0".to_string(),
            json!({
                "type": "Chain",
                "id": "Chain:0",
                "dummy": "0x00",
            }),
        ),
        Node(
            "Block:0".to_string(),
            json!({
                "type": "Block",
                "id": "Block:0",
                "number": "0x00",
                "hash": "0x00",
                "chainId": "0",
                "chain": "Chain:0",
            }),
        ),
    ]);
    schema = schema.data(resolver);
    let schema = schema.finish();
    assert_matches!(schema, Ok(_));
    let schema = schema.unwrap();

    // Execute a query
    let response = execute_query(
        &schema,
        r#"
          query {
              node(id: "Block:0") {
                  id
                  ... on Block {
                    hash
                    chain {
                        id
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
        "id": "Block:0",
        "hash": "0x00",
        "chain": {
          "id": "Chain:0"
        }
      }
    }
    "###
    );
}

#[tokio::test]
async fn sdl() {
    // Build the schema
    let schema = build_schema();
    let schema = schema.finish();
    assert_matches!(schema, Ok(_));
    let schema = schema.unwrap();

    // Print the schema
    let text = schema.pretty_sdl();
    assert_snapshot!(text);
}
