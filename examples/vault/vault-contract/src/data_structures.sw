library;

pub struct User {
    id: Identity,
    balance: u64,
}

impl User {
    pub fn new(id: Identity) -> Self {
        Self { id, balance: 0 }
    }

    pub fn deposit(ref mut self, amount: u64) {
        self.balance += amount;
    }

    pub fn withdraw(ref mut self, amount: u64) {
        self.balance -= amount;
    }
}
