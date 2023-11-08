use bigdecimal::ToPrimitive;
use fuel_indexer_tests::fixtures::{
    mock_request, setup_indexing_test_components, IndexingTestComponents,
};
use fuel_indexer_types::prelude::*;
use fuel_indexer_utils::uid;
use serde::{Deserialize, Serialize};
use sqlx::{types::BigDecimal, Row};
use std::{collections::HashSet, str::FromStr};

const REVERT_VM_CODE: u64 = 0x0004;
const EXPECTED_CONTRACT_ID: &str =
    "f243849dbbbb53783de7ffc1ec12a1d6a42152b456d1b460e413a097694d247d";
const TRANSFER_BASE_ASSET_ID: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

#[actix_web::test]
async fn test_index_events_with_multiple_args_in_index_handler() {
    let IndexingTestComponents {
        ref node, ref db, ..
    } = setup_indexing_test_components(None).await;

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
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pingentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let ping_value = ping_row.get::<BigDecimal, usize>(1).to_u64().unwrap();
    assert_eq!(ping_value, 12345);

    let pong_row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pongentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let pong_value = pong_row.get::<BigDecimal, usize>(1).to_u64().unwrap();
    assert_eq!(pong_value, 45678);

    let pung_row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pungentity LIMIT 1")
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
async fn test_index_blocks_and_transactions() {
    let IndexingTestComponents {
        ref node, ref db, ..
    } = setup_indexing_test_components(None).await;

    mock_request("/block").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();

    let row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.blockentity ORDER BY timestamp DESC LIMIT 1",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    let _id = row.get::<String, usize>(0);
    let timestamp = row.get::<i64, usize>(2);

    assert!(row.get::<i32, usize>(1).to_u64().unwrap() > 1);
    assert!(timestamp > 0);

    // Check for IndexMetadata
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.indexmetadataentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert!(row.get::<BigDecimal, usize>(1).to_u64().unwrap() > 0);
    assert_eq!(row.get::<i32, usize>(2), 1);
}

#[actix_web::test]
async fn test_index_receipt_types() {
    let IndexingTestComponents {
        ref node, ref db, ..
    } = setup_indexing_test_components(None).await;
    let mut conn = db.pool.acquire().await.unwrap();

    mock_request("/call").await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.callentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let fn_name = row.get::<&str, usize>(5);

    assert_eq!(fn_name, "trigger_pure_function");

    mock_request("/returndata").await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.pungentity LIMIT 1")
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

    mock_request("/log").await;

    let row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.logentity WHERE ra = 8675309 LIMIT 1",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(2).to_u64().unwrap(), 8675309);

    mock_request("/logdata").await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.pungentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let from_buff = Address::from_str(&row.get::<String, usize>(3)).unwrap();
    let addr_buff = Address::from_str(
        "0x532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96",
    )
    .unwrap();

    assert!(row.get::<bool, usize>(2));
    assert_eq!(
        Identity::Address(Address::from(<[u8; 32]>::from(from_buff))),
        Identity::Address(Address::from(<[u8; 32]>::from(addr_buff)))
    );

    mock_request("/scriptresult").await;

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

    mock_request("/transfer").await;

    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.transferentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(3).to_u64().unwrap(), 1); // value is defined in test contract
    assert_eq!(row.get::<&str, usize>(4), TRANSFER_BASE_ASSET_ID);

    mock_request("/transferout").await;

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

    mock_request("/revert").await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.revertentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<&str, usize>(1), EXPECTED_CONTRACT_ID);
    assert_eq!(
        row.get::<BigDecimal, usize>(2).to_u64().unwrap(),
        REVERT_VM_CODE
    );

    mock_request("/panic").await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.panicentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<&str, usize>(1), EXPECTED_CONTRACT_ID);
    assert_eq!(row.get::<i32, usize>(2), 5);

    // First, we mint the contract's native asset...
    mock_request("/mint").await;

    let mut conn = db.pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.mintentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<&str, usize>(1), TRANSFER_BASE_ASSET_ID);
    assert_eq!(row.get::<&str, usize>(2), EXPECTED_CONTRACT_ID);
    assert_eq!(row.get::<BigDecimal, usize>(3).to_u64().unwrap(), 100);

    // ...then we burn it.
    mock_request("/burn").await;

    let mut conn = db.pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.burnentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<&str, usize>(1), TRANSFER_BASE_ASSET_ID);
    assert_eq!(row.get::<&str, usize>(2), EXPECTED_CONTRACT_ID);
    assert_eq!(row.get::<BigDecimal, usize>(3).to_u64().unwrap(), 100);

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

    assert_eq!(
        row.get::<&str, usize>(1),
        "abcdefghijklmnopqrstuvwxyz123456"
    );
}

#[actix_web::test]
async fn test_index_128_bit_integers() {
    let IndexingTestComponents {
        ref node, ref db, ..
    } = setup_indexing_test_components(None).await;

    mock_request("/ping").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();

    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.u16entity LIMIT 1")
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
async fn test_index_optional_types() {
    let IndexingTestComponents {
        ref node, ref db, ..
    } = setup_indexing_test_components(None).await;

    mock_request("/ping").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();

    let id = uid(8675309_i32.to_le_bytes()).to_string();

    let row = sqlx::query(&format!(
        "SELECT * FROM fuel_indexer_test_index1.optionentity WHERE id = '{id}'"
    ))
    .fetch_one(&mut conn)
    .await
    .unwrap();

    let opt_int = row.get::<Option<BigDecimal>, usize>(2);

    assert_eq!(row.get::<String, usize>(0), id);
    assert_eq!(row.get::<BigDecimal, usize>(1).to_u64().unwrap(), 100);
    assert!(opt_int.is_some());

    assert_eq!(opt_int.unwrap().to_u64().unwrap(), 999);

    assert!(row.get::<Option<&str>, usize>(3).is_none());
}

#[actix_web::test]
async fn test_index_tuples() {
    let IndexingTestComponents {
        ref node, ref db, ..
    } = setup_indexing_test_components(None).await;

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

#[derive(Serialize, Deserialize, sqlx::FromRow, sqlx::Decode, Debug, Eq, PartialEq)]
struct VirtualEntity {
    name: Option<String>,
    size: i8,
}

#[derive(Debug, Serialize, Deserialize)]
struct UnionEntity {
    a: Option<u64>,
    b: Option<u64>,
    c: Option<u64>,
    union_type: String,
}

#[derive(sqlx::FromRow, sqlx::Type)]
struct ListFKType {
    id: BigDecimal,
    value: BigDecimal,
}

#[actix_web::test]
async fn test_index_types_for_block_explorer() {
    let IndexingTestComponents {
        ref node, ref db, ..
    } = setup_indexing_test_components(None).await;

    mock_request("/block").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.explorerentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert_eq!(
        row.get::<String, usize>(0),
        uid(8675309_i32.to_le_bytes()).to_string()
    );
    let nonce = row.get::<&str, usize>(1);
    let nonce = hex::decode(nonce).unwrap();
    let mut buff: [u8; 32] = [0u8; 32];
    buff.copy_from_slice(&nonce);
    assert_eq!(Nonce::from(buff), Nonce::default());

    let hexstring = row.get::<&str, usize>(3);
    let hexstring = hex::decode(hexstring).unwrap();

    assert_eq!(hexstring, Bytes::from("hello world!"));

    // Non-indexable types
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.usesvirtualentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert_eq!(row.get::<String, usize>(0), uid([1]).to_string());
    assert_eq!(row.get::<&str, usize>(1), "hello world");

    let entity: VirtualEntity =
        serde_json::from_value(row.get::<serde_json::Value, usize>(2)).unwrap();

    assert_eq!(entity.name, Some("virtual".to_string()));
    assert_eq!(entity.size, 1);

    // Union types
    let row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.indexableunionentity LIMIT 1",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    // Fields are in a different order for these union types
    assert_eq!(row.get::<String, usize>(0), uid([1]).to_string());
    assert_eq!(row.get::<BigDecimal, usize>(1).to_u64().unwrap(), 5);
    assert_eq!(row.get::<&str, usize>(2), "UnionType::A");
    assert_eq!(row.get::<BigDecimal, usize>(3).to_u64().unwrap(), 10);

    let row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.virtualunioncontainerentity LIMIT 1",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    assert_eq!(row.get::<String, usize>(0), uid([1]).to_string());
    let entity: UnionEntity =
        serde_json::from_value(row.get::<serde_json::Value, usize>(1)).unwrap();
    assert_eq!(row.get::<&str, usize>(2), "UnionType::B");

    assert_eq!(entity.a.unwrap(), 2);
    assert!(entity.b.is_none());
    assert_eq!(entity.c.unwrap(), 6);

    // List types
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.listtypeentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let expected_required_all = (1..4)
        .map(|x| uid([x]).to_string())
        .collect::<Vec<String>>()
        .sort();
    assert_eq!(row.get::<String, usize>(0), uid([1]).to_string());
    assert_eq!(row.get::<&str, usize>(1), "hello world");
    assert_eq!(
        row.get::<Vec<String>, usize>(2).sort(),
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
        .map(|x| x.get::<String, usize>(0))
        .collect::<HashSet<String>>();
    assert_eq!(parent_ids.len(), 1);
    assert!(parent_ids.contains(&uid([1]).to_string()));

    // Should have 3 unique child IDs
    let child_ids = row
        .iter()
        .map(|x| x.get::<String, usize>(1))
        .collect::<HashSet<String>>();
    assert_eq!(child_ids.len(), 3);
    assert!(child_ids.contains(&uid([1]).to_string()));
    assert!(child_ids.contains(&uid([2]).to_string()));
    assert!(child_ids.contains(&uid([3]).to_string()));
}

#[actix_web::test]
async fn test_index_sway_enums() {
    let IndexingTestComponents {
        ref node, ref db, ..
    } = setup_indexing_test_components(None).await;

    mock_request("/enum").await;

    let mut conn = db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.complexenumentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert_eq!(row.get::<&str, usize>(1), "EnumEntity::One");

    mock_request("/enum_error").await;

    node.abort();

    let mut conn = db.pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.enumerror LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<&str, usize>(1), EXPECTED_CONTRACT_ID);
    assert_eq!(row.get::<BigDecimal, usize>(2).to_u64().unwrap(), 0);
}

#[actix_web::test]
async fn test_start_block() {
    let IndexingTestComponents {
        ref db,
        ref manifest,
        ..
    } = setup_indexing_test_components(None).await;
    let mut conn = fuel_indexer_database::IndexerConnection::Postgres(Box::new(
        db.pool.acquire().await.unwrap(),
    ));

    // Allow the indexer to start and process blocks.
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let start = fuel_indexer::get_start_block(&mut conn, &manifest)
        .await
        .unwrap();

    // setup_indexing_test_components deploys a contract, so one block exists.
    // The indexer should have processed that block. The start block is
    // therefore 2 (if we started from 1, the indexer would have processed it
    // twice).
    assert_eq!(start, 2);

    // We create two more blocks, 2, and 3, which are processed by the indexer.
    mock_request("/block").await;
    mock_request("/block").await;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let start = fuel_indexer::get_start_block(&mut conn, &manifest)
        .await
        .unwrap();

    // The next start block is therefore 4. The indexer should have processed
    // blocks 1, 2, and 3.
    assert_eq!(start, 4);
}

#[actix_web::test]
async fn test_generics() {
    let IndexingTestComponents { ref db, .. } =
        setup_indexing_test_components(None).await;

    mock_request("/generics").await;

    let mut conn = db.pool.acquire().await.unwrap();
    let expected_id = "4405f11f2e332ea850a884ce208d97d0cd68dc5bc0fd124a1a7b7f99962ff99b";
    let row = sqlx::query(&format!(
        "SELECT * FROM fuel_indexer_test_index1.pingentity where id = '{expected_id}'"
    ))
    .fetch_one(&mut conn)
    .await
    .unwrap();

    assert_eq!(row.get::<&str, usize>(0), expected_id);
    assert_eq!(row.get::<BigDecimal, usize>(1).to_u64().unwrap(), 8888);
    assert_eq!(
        row.get::<&str, usize>(2),
        "aaaasdfsdfasdfsdfaasdfsdfasdfsdf"
    );
}

#[actix_web::test]
async fn test_no_missing_blocks() {
    let IndexingTestComponents {
        ref db,
        ref manifest,
        ..
    } = setup_indexing_test_components(None).await;

    let mut conn = fuel_indexer_database::IndexerConnection::Postgres(Box::new(
        db.pool.acquire().await.unwrap(),
    ));

    // Allow the indexer to start and process blocks.
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    mock_request("/block").await;
    mock_request("/block").await;

    let start = fuel_indexer_database::queries::last_block_height_for_indexer(
        &mut conn,
        &manifest.namespace(),
        &manifest.identifier(),
    )
    .await
    .unwrap();

    assert_eq!(start, 3);

    // Remove the last item from indexmetadataentity, simulating missing a block.
    let mut conn_2 = db.pool.acquire().await.unwrap();
    sqlx::query(
        "DELETE FROM fuel_indexer_test_index1.indexmetadataentity WHERE block_height = 3",
    )
    .execute(&mut conn_2)
    .await
    .unwrap();

    // Trigger more blocks. The indexer will receive blocks 4 and 5. However,
    // due to a missing item from indexmetadataentity, the indexer can't
    // progress.
    mock_request("/block").await;
    mock_request("/block").await;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // last_block_height_for_indexer fetches MAX(block_height) from
    // indexmetadataentity. Thus, if the indexer processed blocks 4 and 5, the
    // value would be 5. Since the DB trigger stopped the indexer from
    // progressing, and since we've deleted one row, the expected value is 2.
    let start = fuel_indexer_database::queries::last_block_height_for_indexer(
        &mut conn,
        &manifest.namespace(),
        &manifest.identifier(),
    )
    .await
    .unwrap();

    assert_eq!(start, 2);
}

#[actix_web::test]
async fn test_find() {
    let IndexingTestComponents {
        ref node, ref db, ..
    } = setup_indexing_test_components(None).await;

    mock_request("/find").await;
    mock_request("/find").await;
    mock_request("/find").await;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let mut conn = db.pool.acquire().await.unwrap();

    let row = sqlx::query("SELECT * FROM index_status")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<&str, usize>(2), "Indexed 4 blocks");
    assert_eq!(row.get::<&str, usize>(1), "running");

    let values = sqlx::query("SELECT * FROM fuel_indexer_test_index1.findentity")
        .fetch_all(&mut conn)
        .await
        .unwrap()
        .iter()
        .map(|r| r.get::<BigDecimal, usize>(1).to_u64().unwrap())
        .collect::<Vec<u64>>();

    assert_eq!(values, vec![2, 1]);

    mock_request("/find").await;
    mock_request("/find").await;

    node.abort();

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let row = sqlx::query("SELECT * FROM index_status")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<&str, usize>(1), "error");
    assert!(row
        .get::<&str, usize>(2)
        .contains("called `Option::unwrap()` on a `None` value"));
}
