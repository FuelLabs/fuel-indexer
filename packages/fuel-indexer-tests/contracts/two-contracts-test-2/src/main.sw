contract;

use std::{address::Address, hash::sha256};

abi Simple {
    fn gimme_some_bar_event(num: u64) -> SomeBarEvent;
}

fn make_someevent(num: u64) -> SomeBarEvent {
    let a_bignum: b256 = sha256(num);
    let fake_account: Address = Address::from(a_bignum);

    SomeBarEvent {
        id: num,
        account: fake_account.into(),
    }
}

struct SomeBarEvent {
    id: u64,
    account: b256,
}

impl Simple for Contract {
    fn gimme_some_bar_event(num: u64) -> SomeBarEvent {
        make_someevent(num)
    }
}
