contract;

use std::logging::log;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        log(1);
        true
    }
}
