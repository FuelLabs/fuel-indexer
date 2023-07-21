extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "examples/scalar-types/scalar-types/scalar_types.manifest.yaml")]
pub mod scalar_types_index_mod {

    fn scalar_types_account_handler(block_data: BlockData) {
        let account = Account {
            id: 1,

            index: 0,

            name: Some("name".to_string()),

            nonce: Nonce::default(),

            balance: 0,

            address: Address::default(),

            metadata: Json::default(),

            transactions: vec![],

            avatar_url: Some("http://gify.com/foo".to_string()),

            updated_at: 1,

            created_at: 1,
        };

        account.save();
    }

    fn scalar_types_wallet_handler(block_data: BlockData) {
        let wallet = Wallet {
            id: 1,

            pubkey: Bytes32::default(),

            block_height: 0.into(),

            balance: 0,

            locked: false,

            accounts: vec![],

            signature: Signature::default(),

            metadata: Json::default(),

            blob: Some(vec![].into()),

            updated_at: 1,

            created_at: 1,
        };

        wallet.save();
    }
}
