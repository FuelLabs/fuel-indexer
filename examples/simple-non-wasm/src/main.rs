extern crate alloc;
use anyhow::Result;
use fuel_executor::{
    CustomHandler, CustomIndexExecutor, GraphQlApi, IndexerConfig, IndexerService, Manifest,
    ReceiptEvent,
};
use fuel_indexer::EntityResult;
use fuel_indexer_derive::graphql_schema;
use fuel_indexer_schema::Address;
use fuels::core::{abi_decoder::ABIDecoder, InvalidOutputType, ParamType, Token, Tokenizable};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
graphql_schema!("simple_handler", "schema/counter.graphql");

// Object being indexed
#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct CountEvent {
    count: u64,
    timestamp: u64,
}

// Implement Tokenizable for object being indexed
impl Tokenizable for CountEvent {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        let arr: Vec<u8> = Vec::from_token(token)?;
        let count = u64::from_be_bytes(arr[..8].try_into().unwrap());
        let timestamp = u64::from_be_bytes(arr[8..].try_into().unwrap());

        Ok(Self { count, timestamp })
    }

    fn into_token(self) -> Token {
        Token::Array(vec![Token::U64(self.count), Token::U64(self.timestamp)])
    }
}

// Implement Entity trait for object being indexed
impl Entity for CountEvent {
    const TYPE_ID: u64 = 0;
    fn from_row(row: Vec<FtColumn>) -> Self {
        let count = match (&row[0]).to_owned() {
            FtColumn::Bytes8(x) => u64::from_le_bytes(*x),
            _ => panic!("Invalid column type"),
        };

        let timestamp = match (&row[1]).to_owned() {
            FtColumn::Bytes8(x) => u64::from_le_bytes(*x),
            _ => panic!("Invalid column type"),
        };

        Self { count, timestamp }
    }

    fn to_row(&self) -> Vec<FtColumn> {
        vec![FtColumn::UInt8(self.count), FtColumn::UInt8(self.timestamp)]
    }

    fn to_vec(&self) -> Vec<u8> {
        let mut result = self.count.to_le_bytes().to_vec();
        let mut timestamp = self.timestamp.to_le_bytes().to_vec();
        result.append(&mut timestamp);
        result
    }
}

fn count_handler(data: Vec<Vec<u8>>) -> Result<Option<EntityResult>> {
    info!("count_handler invoked");
    if let Some(data) = data.first() {
        let tokens = ABIDecoder::new()
            .decode(
                &[
                    ParamType::B256,
                    ParamType::U64,
                    ParamType::U64,
                    ParamType::B256,
                    ParamType::Array(Box::new(ParamType::U8), 16),
                    ParamType::U64,
                    ParamType::U64,
                ],
                data,
            )
            .expect("Bad encoding!");

        let event = CountEvent::from_token(tokens[4].to_owned()).unwrap();

        return Ok(Some(event.as_entity_result()));
    }

    Ok(None)
}

// Another object being indexed
#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct AnotherCountEvent {
    count: u64,
    timestamp: u64,
    address: Address,
}

fn another_count_handler(data: Vec<Vec<u8>>) -> Result<Option<EntityResult>> {
    info!("another_count_handler invoked");
    if let Some(data) = data.first() {
        let tokens = ABIDecoder::new()
            .decode(
                &[
                    ParamType::B256,
                    ParamType::U64,
                    ParamType::U64,
                    ParamType::U64,
                    ParamType::U64,
                    ParamType::B256,
                    ParamType::Array(Box::new(ParamType::U8), 16),
                    ParamType::U64,
                    ParamType::U64,
                ],
                data,
            )
            .expect("Bad Encoding!");

        let event = CountEvent::from_token(tokens[6].to_owned()).unwrap();

        info!("another_count_handler received event: {:?}", event);
    }

    Ok(None)
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

    // Load executor config
    let manifest = Manifest::new(
        "simple_handler".to_owned(),
        "schema/counter.graphql".to_owned(),
        None,
    );

    let schema = manifest.graphql_schema().await?;

    // In this example, we've already started our fuel node on another process
    info!("Fuel node listening on {}", config.fuel_node_addr);
    let api_handle = tokio::spawn(GraphQlApi::run(config.clone()));

    // Create a new service to run
    let mut service = IndexerService::new(config.clone())?;
    let _ = service.build_schema(&manifest, &schema)?;

    // Create a new executor to run on the service
    let mut executor = CustomIndexExecutor::new(&config.database_url, manifest.clone())?;

    // Add some handlers to the executor, in order to process events
    executor
        .handler(CustomHandler::new(
            ReceiptEvent::ReturnData,
            manifest.namespace.clone(),
            "Count".to_owned(),
            count_handler,
        ))
        .handler(CustomHandler::new(
            ReceiptEvent::LogData,
            manifest.namespace.clone(),
            "AnotherCount".to_owned(),
            another_count_handler,
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
