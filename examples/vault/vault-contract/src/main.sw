contract;

mod data_structures;
mod errors;
mod events;

use data_structures::User;
use errors::{AssetError, VaultError};
use events::{CreateEvent, DepositEvent, WithdrawEvent};
use vault_interface::Vault;
use std::{
    call_frames::msg_asset_id,
    constants::BASE_ASSET_ID,
    context::msg_amount,
    token::transfer,
};

configurable {
    ASSET: ContractId = BASE_ASSET_ID,
}

storage {
    vaults: StorageMap<Identity, Option<User>> = StorageMap {},
}

impl Vault for Contract {
    #[storage(read, write)]
    fn create() {
        let caller = msg_sender().unwrap();
        let vault = storage.vaults.get(caller).try_read();

        require(vault.is_none(), VaultError::AlreadyExists);

        let user = User::new(caller);
        storage.vaults.insert(caller, Some(user));

        log(CreateEvent { user });
    }

    #[payable]
    #[storage(read, write)]
    fn deposit() {
        let caller = msg_sender().unwrap();
        let vault = storage.vaults.get(caller).try_read();

        require(vault.is_some(), VaultError::DoesNotExist);
        require(msg_asset_id() == ASSET, AssetError::IncorrectAsset);

        let mut user = vault.unwrap().unwrap();
        user.deposit(msg_amount());
        storage.vaults.insert(caller, Some(user));

        log(DepositEvent {
            id: caller,
            amount: msg_amount(),
        });
    }

    #[storage(read, write)]
    fn withdraw(amount: u64) {
        let caller = msg_sender().unwrap();
        let vault = storage.vaults.get(caller).try_read();

        require(vault.is_some(), VaultError::DoesNotExist);

        let mut user = vault.unwrap().unwrap();
        require(amount <= user.balance, AssetError::InsufficientBalance);

        user.withdraw(amount);
        storage.vaults.insert(caller, Some(user));

        transfer(amount, ASSET, caller);

        log(WithdrawEvent {
            id: caller,
            amount,
        });
    }
}
