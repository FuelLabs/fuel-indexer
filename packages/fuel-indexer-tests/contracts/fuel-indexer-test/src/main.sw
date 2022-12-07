contract;

use std::{
    address::Address,
    call_frames::contract_id,
    constants::BASE_ASSET_ID,
    identity::Identity,
    logging::log,
    message::send_message,
    revert::revert,
    token::transfer,
};

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

abi FuelIndexer {
    fn trigger_multiargs() -> Ping;
    fn trigger_callreturn() -> Pung;
    fn trigger_ping() -> Ping;
    fn trigger_pong() -> Pong;
    fn trigger_transfer();
    fn trigger_log();
    fn trigger_logdata();
    fn trigger_scriptresult();
    fn trigger_transferout();
    fn trigger_messageout();
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

    fn trigger_pong() -> Pong {
        let p = Pong {
            id: 2,
            value: 123,
        };
        p
    }

    fn trigger_transfer() {
        // Transfer the asset back to the originating contract
        transfer(1, BASE_ASSET_ID, Identity::ContractId(contract_id()));
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

    fn trigger_transferout() {
        const RECEIVER = Address::from(0x532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96);
        transfer(1, BASE_ASSET_ID, Identity::Address(RECEIVER));
    }

    fn trigger_messageout() {
        let mut v: Vec<u64> = Vec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        const RECEIVER = 0x532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96;
        send_message(RECEIVER, v, 100);
    }
}
