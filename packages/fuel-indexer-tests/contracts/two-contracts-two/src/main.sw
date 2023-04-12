contract;

use std::{address::Address, hash::sha256};

abi SimpleTwo {
    fn trigger_contract_two(num: u64) -> Bar;
}

fn make_some_event(num: u64) -> Bar {
    Bar {
        num,
    }
}

struct Bar {
    num: u64,
}

impl SimpleTwo for Contract {
    fn trigger_contract_two(num: u64) -> Bar {
        make_some_event(num)
    }
}
