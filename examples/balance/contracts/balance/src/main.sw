contract;

use std::logging::log;
use std::address::Address;

struct BalanceEvent {
    id: u64,
    amount: u64,
    timestamp: u64,
}

abi Balance {
    #[storage(write, read)]
    fn init_balance() -> BalanceEvent;

    #[storage(read)]
    fn get_balance() -> BalanceEvent;

    #[storage(write, read)]
    fn liquidate_balance() -> BalanceEvent;
}

storage {
    balance: u64 = 0,
}

impl Balance for Contract {
    #[storage(write, read)]
    fn init_balance() -> BalanceEvent {
        storage.balance = 100;
        log("Balance initialized");
        
        BalanceEvent {
            id: 1,
            amount: storage.balance,
            timestamp: 500,
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
    fn liquidate_balance() -> BalanceEvent {
        storage.balance = 0;

        log("Balanced liquidated");

        BalanceEvent {
            id: 1,
            amount: storage.balance,
            timestamp: 2000,
        }
    }
}

