extern crate alloc;
use anyhow::Result;
use fuel_indexer::{
    config::{IndexerArgs, IndexerConfig, Parser},
    Address, GraphQlApi, IndexerResult, IndexerService, Manifest, NativeHandlerResult, Receipt,
};
use fuel_indexer_macros::graphql_schema;
use fuels::core::{abi_decoder::ABIDecoder, ParamType, Tokenizable};
use fuels_abigen_macro::abigen;
use tracing::info;
use tracing_subscriber::filter::EnvFilter;

// Load graphql schema
graphql_schema!("counter", "schema/counter.graphql");

// Load structs from abigen
abigen!(
    Counter,
    "examples/simple-native/contracts/counter/out/debug/counter-abi.json"
);

fn count_handler(receipt: Receipt) -> Option<IndexerResult<NativeHandlerResult>> {
    match receipt {
        Receipt::ReturnData { data, .. } => {
            // Define which params we expect (using the counter-abi.json as a reference)
            let params = vec![ParamType::Struct(vec![
                ParamType::U64,
                ParamType::U64,
                ParamType::U64,
            ])];

            // Decode the data into a Token using these params
            let token = ABIDecoder::decode(&params, &data).unwrap();

            // Recover the CountEvent from this token
            let event = CountEvent::from_token(token[0].to_owned()).unwrap();

            // Using the Count entity from the GraphQL schema
            let count = Count {
                id: event.id,
                timestamp: event.timestamp,
                count: event.count,
            };

            Some(Ok(count.pack()))
        }
        _ => None,
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    // Uneccessary, but helpful tracing
    let filter = match std::env::var_os("RUST_LOG") {
        Some(_) => EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided"),
        None => EnvFilter::new("info"),
    };

    tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    let opt = IndexerArgs::from_args();

    // Load the node config
    let config = match &opt.config {
        Some(path) => IndexerConfig::from_file(path).await?,
        None => IndexerConfig::from_opts(opt.clone()),
    };

    // Load the indexer manifest
    let manifest = Manifest::from_file(&opt.manifest.unwrap())?;

    // Create a new service to run
    let mut service = IndexerService::new(config.clone()).await?;

    // Add an indexer comprised of a list of handlers
    service
        .add_native_indexer(manifest, false, vec![count_handler])
        .await?;

    // Kick it off!
    let service_handle = tokio::spawn(service.run());

    // In this example, we've already started our fuel node on another process
    info!("Fuel node listening on {}", config.fuel_node.to_string());
    GraphQlApi::run(config.clone()).await;

    service_handle.await?;

    Ok(())
}
