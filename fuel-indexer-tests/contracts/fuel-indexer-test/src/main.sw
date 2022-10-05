contract;

use std::logging::log;

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
}

abi FuelIndexer {
    fn ping() -> Ping;
    fn pong() -> Pong;
    fn pung() -> Pung;
}

impl FuelIndexer for Contract {
    fn ping() -> Ping {
        let p = Ping{
            id: 1,
            value: 123,
            message: "aaaasdfsdfasdfsdfaasdfsdfasdfsdf"
        };
        log("This is a generic log message");
        p
    }

    fn pong() -> Pong {
        let p = Pong{ id: 2, value: 123 };
        p
    }

    fn pung() -> Pung {
        let p = Pung{ id: 1, value: 456 };
        log(p);
        p
    }
}
