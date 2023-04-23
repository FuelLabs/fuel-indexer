use crate::cli::PullAbiCommand;
use anyhow::anyhow;
use reqwest::Url;
use std::{fs::File, io::prelude::*};
use tracing::info;

pub async fn init(command: PullAbiCommand) -> anyhow::Result<()> {
    let PullAbiCommand {
        raw_url,
        contract_name,
        path,
        verbose,
        ..
    } = command;

    let url = Url::parse(&raw_url)?;
    let client = reqwest::Client::new();
    let response = client.get(url.clone()).send().await?;
    let content = response.text().await?;

    let file_name = match contract_name {
        Some(name) => format!("{}-abi.json", name),
        None => url
            .path_segments()
            .ok_or(anyhow!("Invalid URL path"))?
            .last()
            .ok_or(anyhow!("Invalid URL path"))?
            .to_owned(),
    };

    let output_dir = match path {
        Some(p) => p,
        None => std::env::current_dir()?,
    };

    let file_path = output_dir.join(file_name);
    let mut file = File::create(&file_path)?;
    file.write_all(content.as_bytes())?;

    if verbose {
        println!("ABI file saved to: {:?}", file_path);
    }

    info!("✅ ABI file saved to: {:?}", file_path);

    Ok(())
}
