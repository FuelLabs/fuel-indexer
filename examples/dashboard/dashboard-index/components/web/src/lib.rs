pub mod helpers {
    use fuels::prelude::{AssetConfig, AssetId, BASE_ASSET_ID};
    use fuels::test_helpers::WalletsConfig;

    // Generate a configuration for multiple wallets so that the wallets
    // used in the node and API are exactly the same.
    pub fn generate_multi_wallet_config() -> WalletsConfig {
        let num_wallets = 10;
        let num_assets = 10;
        let num_coins = 10;
        let coin_amount = 1_000_000;

        let mut assets = vec![AssetConfig {
            id: BASE_ASSET_ID,
            num_coins,
            coin_amount,
        }];

        for n in 1..num_assets {
            let asset = AssetConfig {
                id: AssetId::new([n; 32]),
                num_coins,
                coin_amount,
            };
            assets.push(asset);
        }

        let wallets_config = WalletsConfig::new_multiple_assets(num_wallets, assets);
        wallets_config
    }
}
