use anyhow::Result;
use api_server::GraphQlApi;
use async_std::{fs::File, io::ReadExt, net::SocketAddr};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;
use tracing_subscriber::filter::EnvFilter;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    /// API Server listen address.
    listen_address: SocketAddr,
    /// Where the data lives.
    database_url: String,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Indexer API Service", about = "Fuel indexer api")]
pub struct Args {
    #[structopt(short, long, help = "API Server config.")]
    config: PathBuf,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let filter = match std::env::var_os("RUST_LOG") {
        Some(_) => EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided"),
        None => EnvFilter::new("info"),
    };

    tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    let opt = Args::from_args();

    let mut file = File::open(opt.config).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    let ServerConfig {
        listen_address,
        database_url,
    } = serde_yaml::from_str(&contents).expect("Bad yaml file");

    let api = GraphQlApi::new(database_url, listen_address);

    let api_handle = tokio::spawn(api.run());

    api_handle.await?;

    Ok(())
}
