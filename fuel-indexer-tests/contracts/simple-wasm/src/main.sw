contract;

use std::{address::Address, hash::sha256};

abi Simple {
    fn gimme_someevent(num: u64) -> SomeEvent;
    fn gimme_anotherevent(num: u64) -> AnotherEvent;
}

fn make_someevent(num: u64) -> SomeEvent {
    let a_bignum: b256 = sha256(num);
    let fake_account: Address = Address::from(a_bignum);

    SomeEvent {
        id: num,
        account: fake_account.into(),
    }
}

struct SomeEvent {
    id: u64,
    account: b256,
}

struct AnotherEvent {
    id: u64,
    account: b256,
    hash: b256,
}

impl Simple for Contract {
    fn gimme_someevent(num: u64) -> SomeEvent {
        make_someevent(num)
    }

    fn gimme_anotherevent(num: u64) -> AnotherEvent {
        let some_event = make_someevent(num);

        AnotherEvent {
            id: num,
            account: some_event.account,
            hash: sha256(num >> 2),
        }
    }
}
