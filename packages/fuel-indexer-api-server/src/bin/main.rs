#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fuel_indexer_api_server::cli::run_cli().await
}
