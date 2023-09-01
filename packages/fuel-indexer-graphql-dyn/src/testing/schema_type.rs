use crate::schema::*;
use crate::store::*;

pub fn new_test_schema_type(store_type: &StoreType) -> Result<DynSchemaType, ()> {
    let mut schema_type = DynSchemaTypeBuilder::new(store_type);

    let (chain_id, chain) = schema_type.define_node("Chain");
    chain.define_connection("blocks", "ChainHasBlock");
    chain.define_connection("transactions", "ChainHasTransaction");

    let (block_id, block) = schema_type.define_node("Block");
    block.define_ref("chain", "chain_id", &chain_id);
    block.define_connection("transactions", "BlockHasTransaction");

    let (_transaction_id, transaction) = schema_type.define_node("Transaction");
    transaction.define_ref("chain", "chain_id", &chain_id);
    transaction.define_ref("block", "block_hash", &block_id);

    let _chain_has_block = schema_type.define_edge(
        "ChainHasBlock",
        "Chain".to_string(),
        "Block".to_string(),
    );
    let _chain_has_transaction = schema_type.define_edge(
        "ChainHasTransaction",
        "Chain".to_string(),
        "Transaction".to_string(),
    );
    let _block_has_transaction = schema_type.define_edge(
        "BlockHasTransaction",
        "Block".to_string(),
        "Transaction".to_string(),
    );

    schema_type.finish()
}

#[test]
fn test_schema_type() {
    use crate::testing::prelude::*;
    use insta::*;

    let store_type = new_test_store_type().unwrap();
    let schema_type = new_test_schema_type(&store_type).unwrap();
    assert_ron_snapshot!(schema_type);
}
