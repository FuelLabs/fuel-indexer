script;

use vault_interface::Vault;

fn main() {
    let caller = abi(Vault, vault_contract::CONTRACT_ID);
    caller.create();
}
