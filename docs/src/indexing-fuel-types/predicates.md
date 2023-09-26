# Predicates

## Definition

- Immutable and ephemeral.
- Predicates are one of the most important abstractions that Fuel makes availabe. A predicate effectively has two states: a _created_ state, and a _spent_ state. A predicate has a `created` state when the predicate is...created.

  - A predicate becomes "created" when a given UTXO inclues certain parameters in its [witness data](./transactions.md).
  - A predicate becomes "spent" when the aforementioned UTXO is used as an input into a subsequent [Transaction](./transactions.md), where this subsequent Transaction also includes certain parameters in its witness data.

> TLDR: A predicate is a UTXO that includes extra information sent in the Transaction.

```rust,ignore
/// The indexer's public representation of a predicate.
///
/// This is a manual copy of `fuels::accounts::predicate::Predicate` due to the
/// fact that `fuels::accounts::predicate::Predicate` is not serializable.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Predicate {
    /// Address of the predicate.
    ///
    /// Using `Address` because `Bech32Address` is not serializable.
    address: Address,

    /// Bytecode of the predicate.
    code: Vec<u8>,

    /// Inputs injected into the predicate.
    ///
    /// Using `Vec<u8> because `UnresolvedBytes` is not serializable.
    data: Vec<u8>,

    /// Chain ID of the predicate.
    chain_id: u64,
}
```

## Usage

In order to get started indexing Predicates, users will need to:

1. Create a new Predicate project using [`forc`]
2. Build this predicate project using `forc build` in order to get the project JSON ABI and predicate template ID.
3. Add this predicate info to your indexer manifest.
4. Include the appropriate witness data in transactions you wish to flag to your indexer

### Sway Predicate

```sway,ignore
predicate;

enum PairAsset {
    BTC : (),
    ETH : (),
}



impl Eq for PairAsset {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (PairAsset::BTC(_), PairAsset::BTC(_)) => true,
            (PairAsset::ETH(_), PairAsset::ETH(_)) => true,
            _ => false,
        }
    }
}

struct OrderPair {
    bid: PairAsset,
    ask: PairAsset,
}

impl Eq for OrderPair {
    fn eq(self, other: Self) -> bool {
        self.bid == other.bid && self.ask == other.ask
    }
}

configurable {
    AMOUNT: u64 = 1u64,
    PAIR: OrderPair = OrderPair { bid: PairAsset::BTC, ask: PairAsset::ETH },
}

fn main(
    amount: u64,
    pair: OrderPair,
) -> bool {
    amount == AMOUNT && pair == PAIR
}
```

### Predicate Indexer

```rust,ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_spent_predicates(predicates: Predicates, configurables: MyPredicateInputs) {
        let template_id = "0xb16545fd38b82ab5178d79c71ad0ce54712cbdcee22f722b08db278e77d1bcbc";
        if let Some(predicate) = predicates.get(template_id) {
            match configurables.pair.bid {
                OrderPair::BTC => info!("Someone wants to give BTC"),
                OrderPair::ETH => info!("Someone wants to get ETH"),
            }
        }

        info!("No predicates for this indexer found in the transaction");
    }
}
```
