contract;

use std::{
    logging::log,
    address::Address,
    constants::BASE_ASSET_ID,
    revert::revert,
    token::transfer,
    identity::Identity,
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
}

abi FuelIndexer {
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
        const RECEIVER = ~Address::from(0x532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96);
        transfer(1, BASE_ASSET_ID, Identity::Address(RECEIVER));
    }

    fn trigger_log() {
        log(8675309);
    }

    fn trigger_logdata() {
        let p = Pung {
            id: 1,
            value: 456,
            is_pung: true,
        };
        log(p);
    }

    fn trigger_scriptresult() {
        log(0);
    }

    // This should trigger both a TR and TRO instruction
    fn trigger_transferout() {
        const RECEIVER = ~Address::from(0x532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96);
        transfer(1, BASE_ASSET_ID, Identity::Address(RECEIVER));
    }

    fn trigger_messageout() {
        // TODO: Revisit after https://github.com/FuelLabs/sway/issues/2899
        // merges as it adds support for send_message_with_output()
    }
}