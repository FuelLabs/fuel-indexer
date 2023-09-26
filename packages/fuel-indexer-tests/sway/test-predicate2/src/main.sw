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
