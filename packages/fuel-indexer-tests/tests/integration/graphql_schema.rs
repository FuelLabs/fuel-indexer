use fuel_indexer_graphql::{
    arguments::{Filter, FilterType, ParsedValue, QueryParams},
    graphql::*,
    queries::{QueryElement, UserQuery},
};
use fuel_indexer_schema::db::tables::Schema;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

fn generate_schema() -> Schema {
    let t = ["Address", "Bytes32", "ID", "Thing1", "Thing2", "UInt16"]
        .iter()
        .map(|s| s.to_string());
    let types = HashSet::from_iter(t);

    let f1 = HashMap::from_iter(
        [("thing1", "Thing1"), ("thing2", "Thing2")]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string())),
    );
    let f2 = HashMap::from_iter(
        [
            ("id", "ID"),
            ("account", "Address"),
            ("huge_number", "UInt16"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string())),
    );
    let f3 = HashMap::from_iter(
        [("id", "ID"), ("account", "Address"), ("hash", "Bytes32")]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string())),
    );
    let fields = HashMap::from_iter([
        ("Query".to_string(), f1),
        ("Thing1".to_string(), f2),
        ("Thing2".to_string(), f3),
    ]);

    Schema {
        version: "".into(),
        namespace: "test_namespace".to_string(),
        identifier: "index1".to_string(),
        query: "Query".into(),
        types,
        fields,
        schema: "".into(),
        foreign_keys: HashMap::new(),
    }
}

#[test]
fn test_query_builder_parses_correctly() {
    let good_query = r#"
        fragment otherfrag on Thing2 {
            id
        }

        fragment frag1 on Thing2 {
            account
            hash
            ...otherfrag
        }

        query GetThing2 {
            thing2(id: 1234) {
                account
                hash
            }
        }

        query OtherQuery {
            thing2(id: 84848) {
                ...frag1
            }
        }
    "#;

    let schema = generate_schema();

    let query = GraphqlQueryBuilder::new(&schema, good_query);
    assert!(query.is_ok());
    let q = query.expect("It's ok here").build();
    assert!(q.is_ok());
    let q = q.expect("It's ok");

    let expected = vec![
        UserQuery {
            elements: vec![
                QueryElement::Field {
                    key: "account".to_string(),
                    value: "test_namespace_index1.thing2.account".to_string(),
                },
                QueryElement::Field {
                    key: "hash".to_string(),
                    value: "test_namespace_index1.thing2.hash".to_string(),
                },
            ],
            joins: HashMap::new(),
            namespace_identifier: "test_namespace_index1".to_string(),
            entity_name: "thing2".to_string(),
            query_params: QueryParams {
                filters: vec![Filter {
                    fully_qualified_table_name: "test_namespace_index1.thing2"
                        .to_string(),
                    filter_type: FilterType::IdSelection(ParsedValue::Number(1234)),
                }],
                sorts: vec![],
                offset: None,
                limit: None,
            },
            alias: None,
        },
        UserQuery {
            elements: vec![
                QueryElement::Field {
                    key: "account".to_string(),
                    value: "test_namespace_index1.thing2.account".to_string(),
                },
                QueryElement::Field {
                    key: "hash".to_string(),
                    value: "test_namespace_index1.thing2.hash".to_string(),
                },
                QueryElement::Field {
                    key: "id".to_string(),
                    value: "test_namespace_index1.thing2.id".to_string(),
                },
            ],
            joins: HashMap::new(),
            namespace_identifier: "test_namespace_index1".to_string(),
            entity_name: "thing2".to_string(),
            query_params: QueryParams {
                filters: vec![Filter {
                    fully_qualified_table_name: "test_namespace_index1.thing2"
                        .to_string(),
                    filter_type: FilterType::IdSelection(ParsedValue::Number(84848)),
                }],
                sorts: vec![],
                offset: None,
                limit: None,
            },
            alias: None,
        },
    ];

    let result = q.parse(&schema);

    // The underlying parser representation of multiple queries uses a
    // HashMap, which does not guarantee ordering. We can't use a HashSet
    // to assert equality due to HashMaps not implementing the Hash trait,
    // so we just assert that the expected elements are present.
    assert!(result.iter().find(|uq| **uq == expected[0]).is_some());
    assert!(result.iter().find(|uq| **uq == expected[1]).is_some());

    let bad_query = r#"
        fragment frag1 on BadType{
            account
        }

        query GetThing2 {
            thing2(id: 123) {
                ...frag
            }
        }
    "#;

    let query = GraphqlQueryBuilder::new(&schema, bad_query);
    assert!(query.is_ok());
    match query.expect("It's ok here").build() {
        Err(GraphqlError::UnrecognizedType(_)) => (),
        o => panic!("Should have gotten Unrecognized type, got {o:?}",),
    }
}
