#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fuel_indexer::cli::run_cli().await
}
