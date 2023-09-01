use crate::store::*;

pub fn new_test_store_type() -> Result<StoreType, ()> {
    let mut store_type = StoreTypeBuilder::default();

    let (call_receipt, data) = store_type.define_composite("CallReceiptData");
    data.define_field("param1", &DataTypeId::U64);
    data.define_field("param2", &DataTypeId::U64);

    let (return_receipt, data) = store_type.define_composite("ReturnReceiptData");
    data.define_field("val", &DataTypeId::U64);

    let (receipt_type, data) = store_type.define_enum("ReceiptType");
    data.define_variant("CallReceipt", &DataTypeId::Unit);
    data.define_variant("ReturnReceipt", &DataTypeId::Unit);

    let (receipt_data, data) = store_type.define_enum("ReceiptData");
    data.define_variant("CallReceiptData", &call_receipt);
    data.define_variant("ReturnReceiptData", &return_receipt);

    let (chain, obj) = store_type.define_obj("Chain");
    obj.define_field("number", &DataTypeId::U64);

    let (block, obj) = store_type.define_obj("Block");
    obj.define_field("number", &DataTypeId::U64);
    obj.define_field("hash", &DataTypeId::B256);
    obj.define_field("parent_hash", &DataTypeId::B256);

    let (transaction, obj) = store_type.define_obj("Transaction");
    obj.define_field("index", &DataTypeId::U64);
    obj.define_field("hash", &DataTypeId::B256);
    obj.define_field("gas_limit", &DataTypeId::U64);
    obj.define_field("gas_price", &DataTypeId::U64);
    obj.define_field("block_hash", &DataTypeId::B256);

    let (_receipt, obj) = store_type.define_obj("Receipt");
    obj.define_field("type", &receipt_type);
    obj.define_field("data", &receipt_data);

    let (_chain_has_block, _assoc) = store_type.define_assoc(&chain, "Has", &block);
    let (_chain_has_transaction, _assoc) =
        store_type.define_assoc(&chain, "Has", &transaction);
    let (_block_has_transaction, _assoc) =
        store_type.define_assoc(&block, "Has", &transaction);

    store_type.finish()
}

#[test]
fn test_store_type() {
    use insta::*;

    let store_type = new_test_store_type().unwrap();
    assert_ron_snapshot!(store_type);
}
