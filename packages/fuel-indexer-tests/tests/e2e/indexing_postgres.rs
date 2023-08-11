use bigdecimal::ToPrimitive;
use fuel_indexer::IndexerConfig;
use fuel_indexer_tests::fixtures::{
    mock_request, setup_indexing_test_components, IndexingTestComponents,
};
use fuel_indexer_types::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{types::BigDecimal, Row};
use std::{collections::HashSet, str::FromStr};

const REVERT_VM_CODE: u64 = 0x0004;
const EXPECTED_CONTRACT_ID: &str =
    "9ccd23f730a8357508ae2b4c8333769d67faccc06e8831b3cf7b20553da39cf9";
const TRANSFER_BASE_ASSET_ID: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

#[actix_web::test]
async fn test_can_trigger_and_index_events_with_multiple_args_in_index_handler_postgres()
{
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/multiarg").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let block_row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.blockentity ORDER BY height DESC LIMIT 1",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    let height = block_row.get::<i32, usize>(1);
    let timestamp: i64 = block_row.get(2);
    assert!(height >= 1);
    assert!(timestamp > 0);

    let ping_row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pingentity WHERE id = 12345")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let ping_value = ping_row.get::<BigDecimal, usize>(1).to_u64().unwrap();
    assert_eq!(ping_value, 12345);

    let pong_row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pongentity WHERE id = 45678")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let pong_value = pong_row.get::<BigDecimal, usize>(1).to_u64().unwrap();
    assert_eq!(pong_value, 45678);

    let pung_row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pungentity WHERE id = 123")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let from_buff = ContractId::from_str(&pung_row.get::<String, usize>(3)).unwrap();

    let contract_buff = ContractId::from_str(
        "0x322ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96",
    )
    .unwrap();

    assert_eq!(
        Identity::ContractId(ContractId::from(<[u8; 32]>::from(from_buff))),
        Identity::ContractId(ContractId::from(<[u8; 32]>::from(contract_buff))),
    );
}

#[actix_web::test]
async fn test_can_trigger_and_index_callreturn_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/callreturn").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pungentity WHERE id = 3")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let from_buff = Address::from_str(&row.get::<String, usize>(3)).unwrap();
    let addr_buff = Address::from_str(
        "0x532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96",
    )
    .unwrap();

    assert_eq!(12345, row.get::<BigDecimal, usize>(1).to_u64().unwrap());
    assert!(row.get::<bool, usize>(2));
    assert_eq!(
        Identity::Address(Address::from(<[u8; 32]>::from(from_buff))),
        Identity::Address(Address::from(<[u8; 32]>::from(addr_buff)))
    );
}

#[actix_web::test]
async fn test_can_trigger_and_index_blocks_and_transactions_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/block").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();

    let row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.blockentity ORDER BY timestamp DESC LIMIT 1",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    let _id = row.get::<BigDecimal, usize>(0).to_u64().unwrap();
    let timestamp = row.get::<i64, usize>(2);

    assert!(row.get::<i32, usize>(1).to_u64().unwrap() > 1);
    assert!(timestamp > 0);
}

#[actix_web::test]
async fn test_can_trigger_and_index_ping_event_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/ping").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pingentity WHERE id = 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 1);
    assert_eq!(row.get::<BigDecimal, usize>(1).to_u64().unwrap(), 123);

    // Ping also triggers the 128-bit integer test as well
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.u16entity WHERE id = 9999")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert_eq!(
        row.get::<BigDecimal, usize>(1),
        BigDecimal::from_str("340282366920938463463374607431768211454").unwrap()
    );
    assert_eq!(
        row.get::<BigDecimal, usize>(2),
        BigDecimal::from_str("170141183460469231731687303715884105727").unwrap()
    );
}

#[actix_web::test]
async fn test_can_trigger_and_index_transfer_event_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/transfer").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.transferentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(3).to_u64().unwrap(), 1); // value is defined in test contract
    assert_eq!(row.get::<&str, usize>(4), TRANSFER_BASE_ASSET_ID);
}

#[actix_web::test]
async fn test_can_trigger_and_index_log_event_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/log").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.logentity WHERE ra = 8675309 LIMIT 1",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(2).to_u64().unwrap(), 8675309);
}

#[actix_web::test]
async fn test_can_trigger_and_index_logdata_event_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/logdata").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pungentity WHERE id = 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let from_buff = Address::from_str(&row.get::<String, usize>(3)).unwrap();
    let addr_buff = Address::from_str(
        "0x532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96",
    )
    .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(1).to_u64().unwrap(), 456);
    assert!(row.get::<bool, usize>(2));
    assert_eq!(
        Identity::Address(Address::from(<[u8; 32]>::from(from_buff))),
        Identity::Address(Address::from(<[u8; 32]>::from(addr_buff)))
    );
}

#[actix_web::test]
async fn test_can_trigger_and_index_scriptresult_event_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/scriptresult").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.scriptresultentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let expected = hex::decode(row.get::<String, usize>(3))
        .unwrap()
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(",");

    assert!((0..=1).contains(&row.get::<BigDecimal, usize>(1).to_u64().unwrap()));
    assert!(row.get::<BigDecimal, usize>(2).to_u64().unwrap() > 0);
    assert_eq!(expected, "1,1,1,1,1".to_string());
}

#[actix_web::test]
#[ignore]
async fn test_can_trigger_and_index_transferout_event_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/transferout").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.transferoutentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert_eq!(
        row.get::<&str, usize>(2),
        "532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96"
    );
    assert_eq!(row.get::<BigDecimal, usize>(3).to_u64().unwrap(), 1);
    assert_eq!(row.get::<&str, usize>(4), TRANSFER_BASE_ASSET_ID);
}

#[actix_web::test]
#[ignore]
async fn test_can_trigger_and_index_messageout_event_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/messageout").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.messageoutentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let recipient = row.get::<&str, usize>(3);
    let amount = row.get::<BigDecimal, usize>(4).to_u64().unwrap();
    assert_eq!(
        recipient,
        "532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96"
    );
    assert_eq!(amount, 100);

    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.messageentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 1234);
    assert_eq!(
        row.get::<&str, usize>(1),
        "abcdefghijklmnopqrstuvwxyz123456"
    );
}

#[actix_web::test]
async fn test_can_index_event_with_optional_fields_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/optionals").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.optionentity WHERE id = 8675309",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    let opt_int = row.get::<Option<BigDecimal>, usize>(2);

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 8675309);
    assert_eq!(row.get::<BigDecimal, usize>(1).to_u64().unwrap(), 100);
    assert!(opt_int.is_some());

    assert_eq!(opt_int.unwrap().to_u64().unwrap(), 999);

    assert!(row.get::<Option<&str>, usize>(3).is_none());
}

#[actix_web::test]
async fn test_can_index_metadata_when_indexer_macro_is_called_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/block").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.indexmetadataentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert!(row.get::<BigDecimal, usize>(1).to_u64().unwrap() > 0);
    assert_eq!(row.get::<i32, usize>(2), 1);
}

#[actix_web::test]
async fn test_can_trigger_and_index_tuple_events_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/tuples").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.tupleentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<&str, usize>(1), "abcde");
    assert_eq!(row.get::<BigDecimal, usize>(2).to_u64().unwrap(), 54321);
    assert_eq!(row.get::<&str, usize>(3), "hello world!");
}

#[actix_web::test]
async fn test_can_trigger_and_index_revert_event_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/revert").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.revertentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 123);
    assert_eq!(row.get::<&str, usize>(1), EXPECTED_CONTRACT_ID);
    assert_eq!(
        row.get::<BigDecimal, usize>(2).to_u64().unwrap(),
        REVERT_VM_CODE
    );
}

#[actix_web::test]
async fn test_can_trigger_and_index_panic_event_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/panic").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.panicentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 123);
    assert_eq!(row.get::<&str, usize>(1), EXPECTED_CONTRACT_ID);
    assert_eq!(row.get::<i32, usize>(2), 5);
}

#[actix_web::test]
async fn test_can_trigger_and_index_enum_error_function_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/enum_error").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.enumerror LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 42);
    assert_eq!(row.get::<&str, usize>(1), EXPECTED_CONTRACT_ID);
    assert_eq!(row.get::<BigDecimal, usize>(2).to_u64().unwrap(), 0);
}

#[actix_web::test]
async fn test_can_trigger_and_index_block_explorer_types_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/block").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.explorerentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 8675309);
    let nonce = row.get::<&str, usize>(1);
    let nonce = hex::decode(nonce).unwrap();
    let mut buff: [u8; 32] = [0u8; 32];
    buff.copy_from_slice(&nonce);
    assert_eq!(Nonce::from(buff), Nonce::default());

    let hexstring = row.get::<&str, usize>(3);
    let hexstring = hex::decode(hexstring).unwrap();

    assert_eq!(hexstring, HexString::from("hello world!"));
}

#[actix_web::test]
async fn test_can_trigger_and_index_enum_types_postgres() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/enum").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.complexenumentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 1);
    assert_eq!(row.get::<&str, usize>(1), "EnumEntity::One");
}

#[derive(Serialize, Deserialize, sqlx::FromRow, sqlx::Decode, Debug, Eq, PartialEq)]
struct VirtualEntity {
    name: Option<String>,
    size: i8,
}

#[actix_web::test]
async fn test_can_trigger_and_index_nonindexable_events() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/block").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.usesvirtualentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 1);
    assert_eq!(row.get::<&str, usize>(1), "hello world");

    let entity: VirtualEntity =
        serde_json::from_value(row.get::<serde_json::Value, usize>(2)).unwrap();

    assert_eq!(entity.name, Some("virtual".to_string()));
    assert_eq!(entity.size, 1);
}

// FIXME: This is not an indexing test...
#[actix_web::test]
async fn test_redeploying_an_already_active_indexer_returns_error_when_replace_indexer_is_false(
) {
    let config = IndexerConfig {
        replace_indexer: false,
        ..IndexerConfig::default()
    };

    let IndexingTestComponents {
        node,
        mut service,
        manifest,
        db: _db,
        ..
    } = setup_indexing_test_components(Some(config)).await;

    node.abort();

    // Attempt to re-register the indexer
    let result = service.register_indexer_from_manifest(manifest).await;

    assert!(result.is_err());

    match result.unwrap_err() {
        fuel_indexer::IndexerError::Unknown(msg) => {
            assert_eq!(&msg, "Indexer(fuel_indexer_test.index1) already exists.")
        }
        err => {
            panic!("Expected Unknown but got: {}", err)
        }
    }
}

// FIXME: This is not an indexing test...
#[actix_web::test]
async fn test_redeploying_an_already_active_indexer_works_when_replace_indexer_is_true() {
    let config = IndexerConfig {
        replace_indexer: true,
        ..IndexerConfig::default()
    };

    let IndexingTestComponents {
        node,
        mut service,
        db,
        manifest,
        ..
    } = setup_indexing_test_components(Some(config)).await;

    // Re-register the indexer
    let _ = service.register_indexer_from_manifest(manifest).await;

    mock_request("/enum").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.complexenumentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 1);
    assert_eq!(row.get::<&str, usize>(1), "EnumEntity::One");
}

#[derive(Debug, Serialize, Deserialize)]
struct UnionEntity {
    a: Option<u64>,
    b: Option<u64>,
    c: Option<u64>,
    union_type: String,
}

#[actix_web::test]
async fn test_can_trigger_and_index_union_types() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/block").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.indexableunionentity LIMIT 1",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    // Fields are in a different order for these union types
    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 1);
    assert_eq!(row.get::<BigDecimal, usize>(1).to_u64().unwrap(), 5);
    assert_eq!(row.get::<&str, usize>(2), "UnionType::A");
    assert_eq!(row.get::<BigDecimal, usize>(3).to_u64().unwrap(), 10);

    let row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.virtualunioncontainerentity LIMIT 1",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 1);
    let entity: UnionEntity =
        serde_json::from_value(row.get::<serde_json::Value, usize>(1)).unwrap();
    assert_eq!(row.get::<&str, usize>(2), "UnionType::B");

    assert_eq!(entity.a.unwrap(), 2);
    assert!(entity.b.is_none());
    assert_eq!(entity.c.unwrap(), 6);
}

#[derive(sqlx::FromRow, sqlx::Type)]
struct ListFKType {
    id: BigDecimal,
    value: BigDecimal,
}

#[actix_web::test]
async fn test_can_trigger_and_index_list_types() {
    let IndexingTestComponents { node, db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/block").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.listtypeentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let expected_required_all = vec![1, 2, 3];
    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 1);
    assert_eq!(row.get::<&str, usize>(1), "hello world");
    assert_eq!(
        row.get::<Vec<BigDecimal>, usize>(2)
            .iter()
            .map(|x| x.to_u64().unwrap())
            .collect::<Vec<u64>>(),
        expected_required_all
    );

    let expected_optional_inner = vec!["hello".to_string(), "world".to_string()];
    assert_eq!(row.get::<Vec<String>, usize>(3), expected_optional_inner);

    let optional_outer = vec![1, 2, 3, 4, 5];
    assert_eq!(
        row.get::<Vec<BigDecimal>, usize>(4)
            .iter()
            .map(|x| x.to_u64().unwrap())
            .collect::<Vec<u64>>(),
        optional_outer
    );

    let optional_all = vec![1, 3];
    assert_eq!(
        row.get::<Vec<BigDecimal>, usize>(5)
            .iter()
            .map(|x| x.to_u64().unwrap())
            .collect::<Vec<u64>>(),
        optional_all
    );

    let expected_virtual_optional_inner = vec![
        Some(VirtualEntity {
            name: Some("foo".to_string()),
            size: 1,
        }),
        Some(VirtualEntity {
            name: Some("bar".to_string()),
            size: 2,
        }),
    ];

    assert_eq!(
        row.get::<Vec<serde_json::Value>, usize>(6)
            .iter()
            .map(|x| {
                Some(serde_json::from_value::<VirtualEntity>(x.to_owned()).unwrap())
            })
            .collect::<Vec<Option<VirtualEntity>>>(),
        expected_virtual_optional_inner
    );

    // Check that data is in M2M table
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.listtypeentitys_listfktypes")
            .fetch_all(&mut conn)
            .await
            .unwrap();
    assert_eq!(row.len(), 3);

    // Should all have the same parent ID
    let parent_ids = row
        .iter()
        .map(|x| x.get::<BigDecimal, usize>(0).to_u64().unwrap())
        .collect::<HashSet<u64>>();
    assert_eq!(parent_ids.len(), 1);
    assert!(parent_ids.contains(&1));

    // Should have 3 unique child IDs
    let child_ids = row
        .iter()
        .map(|x| x.get::<BigDecimal, usize>(1).to_u64().unwrap())
        .collect::<HashSet<u64>>();
    assert_eq!(child_ids.len(), 3);
    assert!(child_ids.contains(&1));
    assert!(child_ids.contains(&2));
    assert!(child_ids.contains(&3));
}
