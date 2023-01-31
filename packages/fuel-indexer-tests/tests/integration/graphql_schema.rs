use fuel_indexer_schema::db::{graphql::*, tables::Schema};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

fn generate_schema() -> Schema {
    let t = ["Address", "Bytes32", "ID", "Thing1", "Thing2"]
        .iter()
        .map(|s| s.to_string());
    let types = HashSet::from_iter(t);

    let f1 = HashMap::from_iter(
        [("thing1", "Thing1"), ("thing2", "Thing2")]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string())),
    );
    let f2 = HashMap::from_iter(
        [("id", "ID"), ("account", "Address")]
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
    }
}

#[test]
fn test_query_builder_generates_proper_sql() {
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

        {
            thing1(id: 4321) { account }
        }
    "#;

    let schema = generate_schema();

    let query = GraphqlQueryBuilder::new(&schema, good_query);
    assert!(query.is_ok());
    let q = query.expect("It's ok here").build();
    assert!(q.is_ok());
    let q = q.expect("It's ok");
    let sql = q.as_sql(true);

    assert_eq!(
        vec![
            "SELECT row_to_json(x) as row from (SELECT account, hash FROM test_namespace.thing2 WHERE id = 1234) x".to_string(),
            "SELECT row_to_json(x) as row from (SELECT account, hash, id FROM test_namespace.thing2 WHERE id = 84848) x".to_string(),
            "SELECT row_to_json(x) as row from (SELECT account FROM test_namespace.thing1 WHERE id = 4321) x".to_string(),
        ],
        sql
    );

    let sql = q.as_sql(false);

    assert_eq!(
        vec![
            "SELECT account, hash FROM test_namespace.thing2 WHERE id = 1234".to_string(),
            "SELECT account, hash, id FROM test_namespace.thing2 WHERE id = 84848"
                .to_string(),
            "SELECT account FROM test_namespace.thing1 WHERE id = 4321".to_string(),
        ],
        sql
    );

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
