contract;

use std::{address::Address, hash::sha256};

abi Simple {
    fn gimme_some_foo_event(num: u64) -> SomeFooEvent;
}

fn make_someevent(num: u64) -> SomeFooEvent {
    let a_bignum: b256 = sha256(num);
    let fake_account: Address = Address::from(a_bignum);

    SomeFooEvent {
        id: num,
        account: fake_account.into(),
    }
}

struct SomeFooEvent {
    id: u64,
    account: b256,
}

impl Simple for Contract {
    fn gimme_some_foo_event(num: u64) -> SomeFooEvent {
        make_someevent(num)
    }
}
