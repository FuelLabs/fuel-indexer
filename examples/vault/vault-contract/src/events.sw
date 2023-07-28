library;

use ::data_structures::User;

pub struct CreateEvent {
    user: User,
}

pub struct DepositEvent {
    id: Identity,
    amount: u64,
}

pub struct WithdrawEvent {
    id: Identity,
    amount: u64,
}
