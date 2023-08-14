//! `async_graphql::dynamic` extensions for handling GraphQL nodes and connections.

pub use super::connection::*;
use async_graphql::dynamic::SchemaBuilder;
use extension_trait::extension_trait;

#[extension_trait]
pub impl SchemaBuilderGraphExt for SchemaBuilder {
    fn register_graph_types(self) -> Self {
        self.register_connection_types()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::test::*;

    #[test]
    pub fn graph_sdl() {
        let mut schema = Schema::test_build().register_graph_types();
        let mut query = Object::new("Query");

        // Some nodes
        let block = Object::new_node("Block")
            .field(Field::new(
                "number",
                TypeRef::named_nn(TypeRef::INT),
                |_| unimplemented!(),
            ))
            .field(Field::new(
                "hash",
                TypeRef::named_nn(TypeRef::STRING),
                |_| unimplemented!(),
            ));
        let transaction = Object::new_node("Transaction")
            .field(Field::new(
                "index",
                TypeRef::named_nn(TypeRef::INT),
                |_| unimplemented!(),
            ))
            .field(Field::new(
                "hash",
                TypeRef::named_nn(TypeRef::STRING),
                |_| unimplemented!(),
            ));

        // Filtering inputs for the nodes
        let int_filter = InputObject::new_ord_filter(TypeRef::INT);
        let string_filter = InputObject::new_ord_filter(TypeRef::STRING);
        let block_filter = InputObject::new_filter("Block")
            .field(InputValue::new(
                "number",
                TypeRef::named(TypeRef::filter_input(TypeRef::INT)),
            ))
            .field(InputValue::new(
                "hash",
                TypeRef::named(TypeRef::filter_input(TypeRef::STRING)),
            ));
        let transaction_filter = InputObject::new_filter("Transaction")
            .field(InputValue::new(
                "index",
                TypeRef::named(TypeRef::filter_input(TypeRef::INT)),
            ))
            .field(InputValue::new(
                "hash",
                TypeRef::named(TypeRef::filter_input(TypeRef::STRING)),
            ));

        // Ordering inputs for the nodes
        let block_order = InputObject::new_order("Block")
            .field(InputValue::new(
                "number",
                TypeRef::named(TypeRef::ORDER_DIRECTION),
            ))
            .field(InputValue::new(
                "hash",
                TypeRef::named(TypeRef::ORDER_DIRECTION),
            ));
        let transaction_order = InputObject::new_order("Transaction")
            .field(InputValue::new(
                "index",
                TypeRef::named(TypeRef::ORDER_DIRECTION),
            ))
            .field(InputValue::new(
                "hash",
                TypeRef::named(TypeRef::ORDER_DIRECTION),
            ));

        // Query fields
        let query_block_field =
            Field::new("block", TypeRef::named("Block"), |_| unimplemented!())
                .argument(InputValue::new("id", TypeRef::named_nn(TypeRef::ID)));
        let query_block_edge = Object::new_edge("Block");
        let query_block_connection = Object::new_connection("Block");
        let query_blocks_field = Field::new(
            "blocks",
            TypeRef::named_nn(TypeRef::connection("Block")),
            |_| unimplemented!(),
        )
        .connection_arguments("Block");
        let query_transaction_field = Field::new(
            "transaction",
            TypeRef::named("Transaction"),
            |_| unimplemented!(),
        )
        .argument(InputValue::new("id", TypeRef::named_nn(TypeRef::ID)));
        let query_transaction_edge = Object::new_edge("Transaction");
        let query_transaction_connection = Object::new_connection("Transaction");
        let query_transactions_field = Field::new(
            "transactions",
            TypeRef::named_nn(TypeRef::connection("Transaction")),
            |_| unimplemented!(),
        )
        .connection_arguments("Transaction");

        query = query
            .field(query_block_field)
            .field(query_blocks_field)
            .field(query_transaction_field)
            .field(query_transactions_field);
        schema = schema
            .register(query_block_edge)
            .register(query_block_connection)
            .register(query_transaction_edge)
            .register(query_transaction_connection);

        schema = schema
            .register(block)
            .register(transaction)
            .register(int_filter)
            .register(string_filter)
            .register(block_filter)
            .register(transaction_filter)
            .register(block_order)
            .register(transaction_order)
            .register(query);

        // Build the schema
        let schema = schema.finish();
        assert_matches!(schema, Ok(_));
        let schema = schema.unwrap();

        // Print the schema
        let text = schema.pretty_sdl();
        assert_snapshot!(text, @r###"
        type Block implements Node {
          id: ID!
          number: Int!
          hash: String!
        }

        type BlockConnection {
          totalCount: Int!
          nodes: [Block!]!
          edges: [BlockEdge!]!
          pageInfo: PageInfo!
        }

        type BlockEdge {
          node: Block!
          cursor: String!
        }

        input BlockFilterInput {
          and: [BlockFilterInput!]
          or: [BlockFilterInput!]
          not: [BlockFilterInput!]
          number: IntFilterInput
          hash: StringFilterInput
        }

        input BlockOrderInput {
          number: OrderDirection
          hash: OrderDirection
        }

        input IDFilterInput {
          and: [IDFilterInput!]
          or: [IDFilterInput!]
          not: [IDFilterInput!]
          eq: IDFilterInput
          in: [IDFilterInput!]
        }

        input IntFilterInput {
          and: [IntFilterInput!]
          or: [IntFilterInput!]
          not: [IntFilterInput!]
          eq: IntFilterInput
          in: [IntFilterInput!]
          gt: IntFilterInput
          gte: IntFilterInput
          lt: IntFilterInput
          lte: IntFilterInput
        }

        interface Node {
          id: ID!
        }

        input NodeFilterInput {
          and: [NodeFilterInput!]
          or: [NodeFilterInput!]
          not: [NodeFilterInput!]
          id: IDFilterInput
        }

        input NodeOrderInput {
          id: OrderDirection
        }

        enum OrderDirection {
          asc
          desc
        }

        type PageInfo {
          hasNextPage: Boolean!
          hasPreviousPage: Boolean!
          startCursor: String
          endCursor: String
        }

        type Query {
          block(id: ID!): Block
          blocks(filter: BlockFilterInput, order: BlockOrderInput, first: Int, after: String, last: Int, before: String): BlockConnection!
          transaction(id: ID!): Transaction
          transactions(filter: TransactionFilterInput, order: TransactionOrderInput, first: Int, after: String, last: Int, before: String): TransactionConnection!
        }

        input StringFilterInput {
          and: [StringFilterInput!]
          or: [StringFilterInput!]
          not: [StringFilterInput!]
          eq: StringFilterInput
          in: [StringFilterInput!]
          gt: StringFilterInput
          gte: StringFilterInput
          lt: StringFilterInput
          lte: StringFilterInput
        }

        type Transaction implements Node {
          id: ID!
          index: Int!
          hash: String!
        }

        type TransactionConnection {
          totalCount: Int!
          nodes: [Transaction!]!
          edges: [TransactionEdge!]!
          pageInfo: PageInfo!
        }

        type TransactionEdge {
          node: Transaction!
          cursor: String!
        }

        input TransactionFilterInput {
          and: [TransactionFilterInput!]
          or: [TransactionFilterInput!]
          not: [TransactionFilterInput!]
          index: IntFilterInput
          hash: StringFilterInput
        }

        input TransactionOrderInput {
          index: OrderDirection
          hash: OrderDirection
        }

        schema {
          query: Query
        }
        "###);
    }
}
