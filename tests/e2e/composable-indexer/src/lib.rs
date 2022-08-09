use fuels::prelude::TxParameters;

pub fn tx_params() -> TxParameters {
    let gas_price = 0;
    let gas_limit = 1_000_000;
    let byte_price = 0;
    TxParameters::new(Some(gas_price), Some(gas_limit), Some(byte_price), None)
}

pub mod defaults {
    use std::time::Duration;

    pub const FUEL_NODE_ADDR: &str = "0.0.0.0:4000";
    pub const FUEL_NODE_HOST: &str = "0.0.0.0";
    pub const FUEL_NODE_PORT: &str = "4000";
    pub const WEB_API_ADDR: &str = "0.0.0.0:8000";
    pub const PING_CONTRACT_ID: &str =
        "68518c3ba3768c863e0d945aa18249f9516d3aa1338083ba79467aa393de109c";
    pub const SLEEP: Duration = Duration::from_secs(60 * 60 * 10);
    pub const WALLET_PASSWORD: &str = "password";
}
