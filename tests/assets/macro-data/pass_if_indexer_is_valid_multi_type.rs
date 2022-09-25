extern crate alloc;
use fuel_indexer_macros::indexer;

#[no_mangle]
fn ff_log_data(_inp: ()) {}


#[indexer(
    abi = "./../tests/assets/contracts/simple_wasm/out/debug/contracts-abi.json",
    namespace = "test_namespace",
    identifier = "index2",
    schema = "./../tests/assets/schema/simple_wasm.graphql",
)]
mod indexer {
    fn function_one(event: SomeEvent) {
        let SomeEvent { id, account } = event;

        assert_eq!(id, 9);
        assert_eq!(account, [48u8; 32]);
    }

    fn function_two(_event: SomeEvent, event2: AnotherEvent) {
        let AnotherEvent { id, account, hash } = event2;

        assert_eq!(id, 9);
        assert_eq!(account, [48u8; 32]);
        assert_eq!(hash, [56u8; 32]);
    }
}

fn main() {
    use fuels_core::{abi_encoder::ABIEncoder, Tokenizable};

    let s1 = SomeEvent {
        id: 9,
        account: [48u8; 32],
    };

    let s2 = AnotherEvent {
        id: 9,
        account: [48u8; 32],
        hash: [56u8; 32],
    };

    let bytes1 = ABIEncoder::new().encode(&[s1.into_token()]).expect("Failed compile test");
    let bytes2 = ABIEncoder::new().encode(&[s2.into_token()]).expect("Failed compile test");

    let data: Vec<BlockData> = vec![
        BlockData {
            height: 0,
            transactions: vec![
                vec![
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
                ]
            ]
        }
    ];

    let mut bytes = serialize(&data);

    let ptr = bytes.as_mut_ptr();
    let len = bytes.len();

    handle_events(ptr, len);
}