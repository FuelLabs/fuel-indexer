extern crate alloc;
use anyhow::Result;
use fuel_executor::{
    CustomHandler, CustomIndexExecutor, Database, GraphQlApi, IndexerConfig, IndexerResult,
    IndexerService, Manifest, ReceiptEvent,
};
use fuel_indexer_derive::graphql_schema;
use fuel_indexer_schema::Address;
use fuels::core::{abi_decoder::ABIDecoder, ParamType, Tokenizable};
use fuels_abigen_macro::abigen;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use tokio::join;
use tracing::{error, info};
use tracing_subscriber::filter::EnvFilter;

#[derive(StructOpt)]
#[structopt(
    name = "Indexer Service",
    about = "Standalone binary for the fuel indexer service"
)]
pub struct Args {
    #[structopt(parse(from_os_str), help = "Indexer service config file")]
    config: PathBuf,
}

// Load graphql schema
graphql_schema!("counter", "schema/counter.graphql");

// Load structs from abigen
abigen!(
    Counter,
    "examples/simple-non-wasm/programs/counter/out/debug/counter-abi.json"
);

fn count_handler(data: Vec<u8>, pg: Arc<Mutex<Database>>) -> IndexerResult<()> {
    // Define which params we expect (using the counter-abi.json as a reference)
    let params = ParamType::Struct(vec![ParamType::U64, ParamType::U64, ParamType::U64]);

    // Decode the data into a Token using these params
    let token = ABIDecoder::decode_single(&params, &data).unwrap();

    // Recover the CountEvent from this token
    let event = CountEvent::from_token(token).unwrap();

    // Using the Count entity from the GraphQL schema
    let count = Count {
        id: event.id,
        timestamp: event.timestamp,
        count: event.count,
    };

    // Save the entity
    pg.lock()
        .expect("Lock poisoned in handler")
        .put_object(count.type_id(), count.to_row(), data);

    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<()> {
    // Uneccessary, but helpful
    let filter = match std::env::var_os("RUST_LOG") {
        Some(_) => EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided"),
        None => EnvFilter::new("info"),
    };

    tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    let opt = Args::from_args();

    // Load node config
    let config: IndexerConfig = IndexerConfig::from_file(&opt.config).await?;

    // Create an executor config
    let manifest = Manifest::new(
        "counter".to_owned(),
        "schema/counter.graphql".to_owned(),
        None,
    );

    let schema = manifest.graphql_schema().await?;

    // In this example, we've already started our fuel node on another process
    info!("Fuel node listening on {}", config.fuel_node_addr);
    let api_handle = tokio::spawn(GraphQlApi::run(config.clone()));

    // Create a new service to run
    let mut service = IndexerService::new(config.clone())?;
    service.build_schema(&manifest, &schema)?;

    // Create a new executor to run on the service
    let mut executor = CustomIndexExecutor::new(&config.database_url, manifest.clone())?;

    // Add some handlers to the executor, in order to process events
    executor.register(CustomHandler::new(
        ReceiptEvent::ReturnData,
        manifest.namespace.clone(),
        count_handler,
    ));

    // Add the executor to our service
    if let Err(e) = service.add_executor(executor, "an_unused_field", manifest, false) {
        panic!("Error: {}", e);
    }

    // Kick it off!
    let service_handle = tokio::spawn(service.run());

    let (first, second) = join!(api_handle, service_handle);

    if let Err(e) = first {
        error!("{:?}", e)
    }
    if let Err(e) = second {
        error!("{:?}", e)
    }

    Ok(())
}
