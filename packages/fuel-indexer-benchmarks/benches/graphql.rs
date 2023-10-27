use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fuel_indexer_database::DbType;
use fuel_indexer_graphql::graphql::GraphqlQueryBuilder;
use fuel_indexer_lib::graphql::GraphQLSchema;
use fuel_indexer_schema::db::tables::IndexerSchema;

fn build_and_parse_query(c: &mut Criterion) {
    c.bench_function("build_and_parse_query", move |b| {
        b.iter(|| {
            let schema_str = r#"type Block {
                id: ID!
                height: U64!
                hash: Bytes32! @unique
            }
            
            type Transaction {
                id: ID!
                block: Block! @join(on:hash)
                hash: Bytes32! @unique
            }"#
            .to_string();
            let gql_schema = GraphQLSchema::new(black_box(schema_str));
            let indexer_schema = IndexerSchema::new(
                "benchmarking",
                "default_indexer",
                black_box(&gql_schema),
                DbType::Postgres,
            )
            .unwrap();

            let query = r#"query { transaction { id hash block { id hash height } } }"#;
            GraphqlQueryBuilder::new(black_box(&indexer_schema), black_box(query))
                .unwrap()
                .build()
                .unwrap()
                .parse(black_box(&indexer_schema));
        })
    });
}

fn build_and_parse_query_with_args(c: &mut Criterion) {
    c.bench_function("build_and_parse_query_with_args", move |b| {
        b.iter(|| {
            let schema_str = r#"type Block {
                id: ID!
                height: U64!
                hash: Bytes32! @unique
            }
            
            type Transaction {
                id: ID!
                block: Block! @join(on:hash)
                hash: Bytes32! @unique
            }"#
            .to_string();
            let gql_schema = GraphQLSchema::new(black_box(schema_str));
            let indexer_schema = IndexerSchema::new(
                "benchmarking",
                "fuel_explorer",
                black_box(&gql_schema),
                DbType::Postgres,
            )
            .unwrap();

            let query = r#"query {
                transaction(order: { id: asc }, filter: { has: [hash] }) {
                    id
                    hash
                    block(order: { height: desc }, filter: { height: { gt: 5 } }) {
                        id
                        hash
                        height
                    }
                }
            }"#;
            GraphqlQueryBuilder::new(black_box(&indexer_schema), black_box(query))
                .unwrap()
                .build()
                .unwrap()
                .parse(black_box(&indexer_schema));
        })
    });
}

criterion_group!(
    graphql,
    build_and_parse_query,
    build_and_parse_query_with_args
);
criterion_main!(graphql);
