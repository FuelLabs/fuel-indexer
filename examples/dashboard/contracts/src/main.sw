contract;

use std::{
    address::Address,
    call_frames::contract_id,
    constants::BASE_ASSET_ID,
    identity::Identity,
    token::transfer,
};

abi Dashboard {
    fn create_transfer(amount: u64, asset_id: ContractId, address: Address);
}

impl Dashboard for Contract {
    // fn create_preloaded_transfer() {
    //     transfer(100, BASE_ASSET_ID, Identity::ContractId(contract_id()));
    // }
    fn create_transfer(amount: u64, asset_id: ContractId, address: Address) {
        transfer(amount, asset_id, Identity::Address(address));
    }
}
