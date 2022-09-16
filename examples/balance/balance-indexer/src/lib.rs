extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(
    abi = "examples/balance/contracts/balance/out/debug/balance-abi.json",
    namespace = "balance",
    identifier = "index1"
    schema = "../schema/balance.graphql"
)]

mod balance {
    fn balance_handler(event: BalanceEvent) {
        let balance = Balance {
            id: event.id,
            timestamp: event.timestamp,
            amount: event.amount,
        };

        balance.save()
    }
}
