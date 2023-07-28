script;

use std::constants::BASE_ASSET_ID;
use vault_interface::Vault;

fn main() {
    let caller = abi(Vault, vault_contract::CONTRACT_ID);
    caller.withdraw(10);
}
