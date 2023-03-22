extern crate alloc;
use fuel_indexer_macros::indexer;

#[no_mangle]
fn ff_log_data(_inp: ()) {}

#[indexer(
    manifest = "packages/fuel-indexer-tests/trybuild/bad_simple_wasm_graphql.yaml"
)]
mod indexer {
    fn function_one(event: SomeEvent) {
        let SomeEvent { id, account } = event;

        assert_eq!(id, 9);
        assert_eq!(account, [48u8; 32]);
    }
}

fn main() {
    use fuels::types::traits::Tokenizable;
    use fuels_core::abi_encoder::ABIEncoder;

    let s = SomeEvent {
        id: 9,
        account: [48u8; 32],
    };

    let encoded = ABIEncoder::encode(&[s.into_token()]).expect("Failed compile test");
    let bytes = encoded.resolve(0);

    let data: Vec<BlockData> = vec![BlockData {
        id: [0u8; 32].into(),
        time: 1,
        height: 0,
        transactions: vec![TransactionData {
            id: [0u8; 32].into(),
            status: TransactionStatus::default(),
            receipts: vec![
                Receipt::Call {
                    id: [0u8; 32].into(),
                    to: [0u8; 32].into(),
                    amount: 400,
                    asset_id: [0u8; 32].into(),
                    gas: 4,
                    param1: 2048508220,
                    param2: 0,
                    pc: 0,
                    is: 0,
                },
                Receipt::ReturnData {
                    id: [0u8; 32].into(),
                    ptr: 2342143,
                    len: bytes.len() as u64,
                    digest: [0u8; 32].into(),
                    data: bytes,
                    pc: 0,
                    is: 0,
                },
            ],
            transaction: Transaction::default(),
        }],
    }];

    let mut bytes = serialize(&data);

    let ptr = bytes.as_mut_ptr();
    let len = bytes.len();

    handle_events(ptr, len);
}
