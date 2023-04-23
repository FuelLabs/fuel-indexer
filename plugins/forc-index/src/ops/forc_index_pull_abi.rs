use crate::{cli::PullAbiCommand, utils::project_dir_info};
use reqwest::Url;
use std::{fs::File, io::prelude::*, path::PathBuf};

pub async fn init(command: PullAbiCommand) -> anyhow::Result<()> {
    let PullAbiCommand {
        url,
        contract_name,
        path,
        verbose,
        ..
    } = command;

    let url = Url::parse(&url)?;
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let content = response.text().await?;

    let file_path = match path {
        Some(p) => p,
        None => {
            let file_name = "contract_abi.json";
            let current_dir = std::env::current_dir()?;
            current_dir.join(file_name)
        }
    };

    let mut file = File::create(&file_path)?;
    file.write_all(content.as_bytes())?;

    if verbose {
        println!("ABI file saved to: {:?}", file_path);
    }

    Ok(())
}
