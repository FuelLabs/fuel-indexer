contract;

use std::{
    address::Address,
    call_frames::contract_id,
    constants::BASE_ASSET_ID,
    identity::Identity,
    logging::log,
    message::send_typed_message,
    revert::revert,
    token::*,
    bytes::Bytes,
};

pub enum UserError {
    Unauthorized: (),
}

pub struct Pong {
    id: u64,
    value: u64,
}


pub struct Ping {
    id: u64,
    value: u64,
    message: str[32],
}

pub struct Pung {
    id: u64,
    value: u64,
    is_pung: bool,
    pung_from: Identity,
}

pub struct TupleStructItem {
    id: u64,
    arr: [u8; 3],
}

pub struct ComplexTupleStruct {
    data: (u32, (u64, bool, (str[5], TupleStructItem))),
}

pub struct SimpleTupleStruct {
    data: (u32, u64, str[12]),
}

pub struct ExplicitQueryStruct {
    id: u64
}

pub struct SimpleQueryStruct {
    id: u64
}

pub struct ExampleMessageStruct {
    id: u64,
    message: str[32]
}

pub enum SimpleEnum {
    One: (),
    Two: (),
    Three: (),
}

pub enum AnotherSimpleEnum {
    Ping: Ping,
    Pung: Pung,
    Call: SimpleEnum,
}

pub enum NestedEnum {
    Inner: AnotherSimpleEnum
}

abi FuelIndexer {
    fn trigger_multiargs() -> Ping;
    fn trigger_callreturn() -> Pung;
    fn trigger_ping() -> Ping;
    fn trigger_ping_for_optional() -> Ping;
    fn trigger_pong() -> Pong;
    #[payable]
    fn trigger_transfer();
    fn trigger_log();
    fn trigger_logdata();
    fn trigger_scriptresult();
    #[payable]
    fn trigger_transferout();
    #[payable]
    fn trigger_messageout();
    fn trigger_tuple() -> ComplexTupleStruct;
    fn trigger_explicit() -> ExplicitQueryStruct;
    fn trigger_deeply_nested() -> SimpleQueryStruct;
    fn trigger_vec_pong_calldata(v: Vec<u8>);
    fn trigger_vec_pong_logdata();
    fn trigger_pure_function();
    fn trigger_panic() -> u64;
    fn trigger_revert();
    fn trigger_enum_error(num: u64);
    fn trigger_enum() -> AnotherSimpleEnum;
    fn trigger_mint();
    #[payable]
    fn trigger_burn();
    fn trigger_generics() -> Option<Ping>;
}

impl FuelIndexer for Contract {
    fn trigger_multiargs() -> Ping {
        log(Pung {
            id: 123,
            value: 54321,
            is_pung: false,
            pung_from: Identity::ContractId(ContractId::from(0x322ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96)),
        });

        log(Pong { id: 45678, value: 45678 });


        Ping { id: 12345, value: 12345, message: "a multiarg ping entity          " }
    }

    fn trigger_callreturn() -> Pung {
        Pung {
            id: 3,
            value: 12345,
            is_pung: true,
            pung_from: Identity::Address(Address::from(0x532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96)),
        }
    }

    fn trigger_ping() -> Ping {
        let p = Ping {
            id: 1,
            value: 123,
            message: "aaaasdfsdfasdfsdfaasdfsdfasdfsdf",
        };
        p
    }

    fn trigger_ping_for_optional() -> Ping {
        let p = Ping {
            id: 8675309,
            value: 123,
            message: "aaaasdfsdfasdfsdfaasdfsdfasdfsdf",
        };
        p
    }

    fn trigger_pong() -> Pong {
        let p = Pong {
            id: 2,
            value: 123,
        };
        p
    }

    #[payable]
    fn trigger_transfer() {
        // Transfer the asset back to the originating contract
        transfer(Identity::ContractId(contract_id()), BASE_ASSET_ID, 1);
    }

    fn trigger_log() {
        log(8675309);
    }

    fn trigger_logdata() {
        let p = Pung {
            id: 1,
            value: 456,
            is_pung: true,
            pung_from: Identity::Address(Address::from(0x532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96)),
        };
        log(p);
    }

    fn trigger_scriptresult() {
        log(0);
    }

    #[payable]
    fn trigger_transferout() {
        const RECEIVER = Address::from(0x532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96);
        transfer_to_address(RECEIVER, BASE_ASSET_ID, 1);
    }

    #[payable]
    fn trigger_messageout() {
        const RECEIVER = 0x532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96;

        let example = ExampleMessageStruct {
            id: 1234,
            message: "abcdefghijklmnopqrstuvwxyz123456"
        };

        send_typed_message(RECEIVER, example, 100);
    }

    fn trigger_tuple() -> ComplexTupleStruct {
        log(SimpleTupleStruct { data: (4u32, 5u64, "hello world!")});
        ComplexTupleStruct{ data: (1u32, (5u64, true, ("abcde", TupleStructItem { id: 54321, arr: [1u8, 2, 3] })))}
    }

    fn trigger_explicit() -> ExplicitQueryStruct {
        ExplicitQueryStruct { id: 456 }
    }

    fn trigger_deeply_nested() -> SimpleQueryStruct {
        SimpleQueryStruct { id: 789 }
    }

    // NOTE: Keeping this to ensure Vec in ABI JSON is ok, even though we don't support it yet
    fn trigger_vec_pong_calldata(v: Vec<u8>) {
        log("This does nothing as we don't handle CallData. But should implement this soon.");
    }

    fn trigger_vec_pong_logdata() {
        let mut v: Vec<Pong> = Vec::new();
        v.push(Pong{ id: 5555, value: 5555 });
        v.push(Pong{ id: 6666, value: 6666 });
        v.push(Pong{ id: 7777, value: 7777 });
        v.push(Pong{ id: 8888, value: 8888 });
        v.push(Pong{ id: 9999, value: 9999 });
    
        log(v);
    }

    fn trigger_pure_function() {
        let _sum = 1 + 2;
    }

    fn trigger_panic() -> u64 {
        let r0: u64 = 18_446_744_073_709_551_615u64;
        // add r0 & r0 (which is the maximum u64 value) and put the result in r1
        asm(r0: r0, r1) {
            add r1 r0 r0; 
            r1: u64
        }
    }

    fn trigger_revert() {
        assert(1 == 0);
    }

    fn trigger_enum_error(num: u64) {
        require(num != 69, UserError::Unauthorized);
    }

    fn trigger_enum() -> AnotherSimpleEnum {
        log(AnotherSimpleEnum::Pung(Pung {
            id: 91231,
            value: 888777,
            is_pung: true,
            pung_from: Identity::ContractId(ContractId::from(0x322ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96)),
        }));

        log(NestedEnum::Inner(AnotherSimpleEnum::Call(SimpleEnum::Three)));

        AnotherSimpleEnum::Ping(Ping {
            id: 7777, 
            value: 7775, 
            message: "hello world! I am, a log event!!"
        })
    }

    fn trigger_mint () {
        mint(BASE_ASSET_ID, 100);
    }

    #[payable]
    fn trigger_burn() {
        burn(BASE_ASSET_ID, 100);
    }

    fn trigger_generics() -> Option<Ping> {
        let x = Some(Ping{ id: 8888, value: 8888, message: "aaaasdfsdfasdfsdfaasdfsdfasdfsdf" });

        let mut v: Vec<Ping> = Vec::new();
        v.push(Ping{ id: 5555, value: 5555, message: "aaaasdfsdfasdfsdfaasdfsdfasdfsdf" });
        v.push(Ping{ id: 6666, value: 6666, message: "aaaasdfsdfasdfsdfaasdfsdfasdfsdf" });
        v.push(Ping{ id: 7777, value: 7777, message: "aaaasdfsdfasdfsdfaasdfsdfasdfsdf" });

        log(x);
        log(v);

        x
    }
}
