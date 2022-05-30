use anyhow::Result;
use async_std::{fs::File, io::ReadExt};
use fuel_indexer_api_server::{Args, GraphQlApi, ServerConfig};
use std::path::PathBuf;
use structopt::StructOpt;
use tracing_subscriber::filter::EnvFilter;

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
