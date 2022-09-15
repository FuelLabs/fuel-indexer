contract;

use std::logging::log;
use std::address::Address;

struct BalanceEvent {
    id: u64,
    timestamp: u64,
    amount: u64,
}

struct AddBalanceEvent {
    id: u64,
    address: b256,
    timestamp: u64,
    amount: u64,
}

abi Balance {
    #[storage(write, read)]
    fn init_balance(value: u64) -> BalanceEvent;

    #[storage(read)]
    fn get_balance() -> BalanceEvent;

    #[storage(write, read)]
    fn increment_balance(sender: b256, amount: u64) -> AddBalanceEvent;
}

storage {
    balance: u64 = 0,
}

impl Balance for Contract {
    #[storage(write, read)]
    fn init_balance(value: u64) -> BalanceEvent {
        storage.balance = value;
        log("Balance initialized");
        
        BalanceEvent {
            id: 1,
            amount: storage.balance,
            timestamp: 1000,
        }
    }

    #[storage(read)]
    fn get_balance() -> BalanceEvent {
        log("Balance retrieved");
        
        BalanceEvent {
            id: 1,
            amount: storage.balance,
            timestamp: 1000,
        }
    }

    #[storage(write, read)]
    fn increment_balance(sender: b256, amount: u64) -> AddBalanceEvent {
        let new_total = storage.balance + amount;
        storage.balance = new_total;

        log("Balanced incremented");

        AddBalanceEvent {
            id: 1,
            address: sender,
            amount: storage.balance,
            timestamp: 1000,
        }
    }
}

