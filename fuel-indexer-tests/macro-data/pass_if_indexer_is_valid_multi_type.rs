extern crate alloc;
use fuel_indexer_macros::indexer;

#[no_mangle]
fn ff_log_data(_inp: ()) {}

#[indexer(
    manifest = "fuel-indexer-tests/assets/macros/simple_wasm.yaml"
)]
mod indexer {
    fn function_one(event: SomeEvent) {
        let SomeEvent { id, account } = event;

        assert_eq!(id, 9);
        assert_eq!(account, Bits256([48u8; 32]));
    }

    fn function_two(_event: SomeEvent, event2: AnotherEvent) {
        let AnotherEvent { id, account, hash } = event2;

        assert_eq!(id, 9);
        assert_eq!(account, Bits256([48u8; 32]));
        assert_eq!(hash, Bits256([56u8; 32]));
    }
}

fn main() {
    use fuels_core::{abi_encoder::ABIEncoder};

    let s1 = SomeEvent {
        id: 9,
        account: Bits256([48u8; 32]),
    };

    let s2 = AnotherEvent {
        id: 9,
        account: Bits256([48u8; 32]),
        hash: Bits256([56u8; 32]),
    };

    let encoded1 = ABIEncoder::encode(&[s1.into_token()]).expect("Failed compile test");
    let bytes1 = encoded1.resolve(0);
    let encoded2 = ABIEncoder::encode(&[s2.into_token()]).expect("Failed compile test");
    let bytes2 = encoded2.resolve(0);


    let data: Vec<BlockData> = vec![BlockData {
        id: [0u8; 32].into(),
        producer: [0u8; 32].into(),
        time: 1,
        height: 0,
        transactions: vec![
            TransactionData {
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
                    len: bytes1.len() as u64,
                    digest: [0u8; 32].into(),
                    data: bytes1,
                    pc: 0,
                    is: 0,
                },
                Receipt::Call {
                    id: [0u8; 32].into(),
                    to: [0u8; 32].into(),
                    amount: 400,
                    asset_id: [0u8; 32].into(),
                    gas: 4,
                    param1: 2379805026,
                    param2: 0,
                    pc: 0,
                    is: 0,
                },
                Receipt::ReturnData {
                    id: [0u8; 32].into(),
                    ptr: 2342143,
                    len: bytes2.len() as u64,
                    digest: [0u8; 32].into(),
                    data: bytes2,
                    pc: 0,
                    is: 0,
                },
            ],
                transaction: Transaction::default(),
            }
        ],
    }];

    let mut bytes = serialize(&data);

    let ptr = bytes.as_mut_ptr();
    let len = bytes.len();

    handle_events(ptr, len);
}
