contract;

use std::{address::Address, hash::sha256};

abi SimpleOne {
    fn trigger_contract_one(num: u64) -> Foo ;
}

fn make_some_event(num: u64) -> Foo {
    Foo {
        num,
    }
}

struct Foo {
    num: u64,
}

impl SimpleOne for Contract {
    fn trigger_contract_one(num: u64) -> Foo {
        make_some_event(num)
    }
}
