contract;

use std::logging::log;
use std::address::Address;

struct CountEvent {
    count: u64,
    timestamp: u64,
}

struct AnotherCountEvent {
    count: u64,
    timestamp: u64,
    address: b256,
}

abi Counter {
    fn init_counter(value: u32) -> CountEvent;
    fn get_count() -> AnotherCountEvent;
    fn increment_counter(value: u32) -> CountEvent;
}

storage {
    counter: u32,
}

impl Counter for Contract {
    fn init_counter(value: u32) -> CountEvent {
        storage.counter = value;
        log("This count was initialized");
        CountEvent {
            count: storage.counter,
            timestamp: 1234567890,
        }
    }

    fn get_count() -> AnotherCountEvent {
        log("This count was retrieved");
        AnotherCountEvent {
            count: storage.counter,
            address: 0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861,
            timestamp: 1234567890,
        }
    }

    fn increment_counter(amount: u32) -> CountEvent {
        let incremented = storage.counter + amount;
        storage.counter = incremented;

        log("This count was incremented");

        CountEvent {
            count: incremented,
            timestamp: 1234567890,
        }
    }
}
