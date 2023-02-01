#![deny(unused_crate_dependencies)]

mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::run_cli().await
}
