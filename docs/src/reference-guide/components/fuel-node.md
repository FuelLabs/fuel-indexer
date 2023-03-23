# Fuel Node

The Fuel indexer indexes data from the Fuel blockchain. Thus, the indexer needs to connect to a Fuel node in order to monitor, process, and save blockchain data. You can connect to either a _local_ Fuel node on your computer or _remote_ Fuel node running on a separate network.

## Local Node

You can run a local Fuel node through the use of `fuel-core`. Once you've installed `fuelup`, you can run `fuel-core run` to start a node. If successful, you can expect to see an output similar to the example below:

```bash
> fuel-core run
2022-12-21T21:23:46.089336Z  INFO fuel_chain_config::config::chain: 63: Initial Accounts
2022-12-21T21:23:46.092146Z  INFO fuel_chain_config::config::chain: 71: PrivateKey(0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c), Address(0x6b63804cfbf9856e68e5b6e7aef238dc8311ec55bec04df774003a2c96e0418e), Balance(10000000)
2022-12-21T21:23:46.092505Z  INFO fuel_chain_config::config::chain: 71: PrivateKey(0x37fa81c84ccd547c30c176b118d5cb892bdb113e8e80141f266519422ef9eefd), Address(0x54944e5b8189827e470e5a8bacfc6c3667397dc4e1eef7ef3519d16d6d6c6610), Balance(10000000)
2022-12-21T21:23:46.092619Z  INFO fuel_chain_config::config::chain: 71: PrivateKey(0x862512a2363db2b3a375c0d4bbbd27172180d89f23f2e259bac850ab02619301), Address(0xe10f526b192593793b7a1559a391445faba82a1d669e3eb2dcd17f9c121b24b1), Balance(10000000)
2022-12-21T21:23:46.092735Z  INFO fuel_chain_config::config::chain: 71: PrivateKey(0x976e5c3fa620092c718d852ca703b6da9e3075b9f2ecb8ed42d9f746bf26aafb), Address(0x577e424ee53a16e6a85291feabc8443862495f74ac39a706d2dd0b9fc16955eb), Balance(10000000)
2022-12-21T21:23:46.092853Z  INFO fuel_chain_config::config::chain: 71: PrivateKey(0x7f8a325504e7315eda997db7861c9447f5c3eff26333b20180475d94443a10c6), Address(0xc36be0e14d3eaf5d8d233e0f4a40b3b4e48427d25f84c460d2b03b242a38479e), Balance(10000000)
2022-12-21T21:23:46.096489Z  WARN fuel_core::cli::run: 158: Fuel Core is using an insecure test key for consensus. Public key: 73dc6cc8cc0041e4924954b35a71a22ccb520664c522198a6d31dc6c945347bb854a39382d296ec64c70d7cea1db75601595e29729f3fbdc7ee9dae66705beb4
2022-12-21T21:23:46.096673Z  INFO fuel_core::cli::run: 212: Fuel Core version v0.14.1
2022-12-21T21:23:46.121974Z  INFO new_node: fuel_core::service::graph_api: 111: Binding GraphQL provider to localhost:4000
```

If you started the node with no additional options (as done in this example), it will be accessible on the default address of `localhost:4000`. The Fuel indexer will attempt to connect to a Fuel node on this address unless instructed otherwise through a configuration file; you can find an [example configuration file](https://github.com/FuelLabs/fuel-indexer/blob/master/config.yaml) in the Fuel indexer repository. To start the indexer service with a custom configuration, you can run `forc index --config [CONFIG_FILE_PATH]`.

## Remote Node

You can also connect the Fuel indexer to a remote Fuel node. To do so, use the example configuration file linked above and change the value for the _Fuel Node configuration_ section to reflect the location of the remote Fuel node. For example:

```yaml
## Fuel Node configuration
#
# fuel_node:
#   host: https://node-beta-3.fuel.network
#   port: 4000
```

You would then start the Fuel indexer service by running `forc index --config [CONFIG_FILE_PATH]`.
