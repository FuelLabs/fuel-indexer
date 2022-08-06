contract;

use std::logging::log;

pub struct Pong {
    id: u64,
    value: u64,
}


pub struct Ping {
    id: u64,
    value: u64,
}

abi Message {
    fn ping() -> Ping;
    fn pong() -> Pong;
}

impl Message for Contract {
    fn ping() -> Ping {
        let p = Ping{ id: 1, value: 123 };
        log(p);
        p
    }

    fn pong() -> Pong {
        let p = Pong{ id: 2, value: 123 };
        log(p);
        p
    }
}
