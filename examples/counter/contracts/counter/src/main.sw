contract;

use std::logging::log;
use std::address::Address;

struct CountEvent {
    id: u64,
    count: u64,
    timestamp: u64,
}

struct AnotherCountEvent {
    id: u64,
    count: u64,
    timestamp: u64,
    address: b256,
}

abi Counter {
    #[storage(write, read)]
    fn init_counter(value: u64) -> CountEvent;

    #[storage(read)]
    fn get_count() -> AnotherCountEvent;

    #[storage(write, read)]
    fn increment_counter(value: u64) -> CountEvent;
}

storage {
    counter: u64 = 0,
}

impl Counter for Contract {
    #[storage(write, read)]
    fn init_counter(value: u64) -> CountEvent {
        storage.counter = value;
        log("This count was initialized");

        CountEvent {
            id: 1,
            count: storage.counter,
            timestamp: 123,
        }
    }

    #[storage(read)]
    fn get_count() -> AnotherCountEvent {
        log("This count was retrieved");

        AnotherCountEvent {
            id: 1,
            count: storage.counter,
            address: 0x068fe90ddc43b18a8f76756ecad8bf30eb0ceea33d2e6990c0185d01b0dbb675,
            timestamp: 123,
        }
    }

    #[storage(write, read)]
    fn increment_counter(amount: u64) -> CountEvent {
        let incremented = storage.counter + amount;
        storage.counter = incremented;

        log("This count was incremented");

        CountEvent {
            id: 1,
            count: incremented,
            timestamp: 123,
        }
    }
}
