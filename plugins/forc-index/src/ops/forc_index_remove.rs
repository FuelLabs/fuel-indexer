use crate::cli::RemoveCommand;
use fuel_indexer_lib::manifest::Manifest;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, AUTHORIZATION},
    StatusCode,
};
use serde_json::{to_string_pretty, value::Value, Map};
use tracing::{error, info};

pub fn init(command: RemoveCommand) -> anyhow::Result<()> {
    let manifest: Manifest = Manifest::from_file(command.manifest.as_path())?;

    let target = format!(
        "{}/api/index/{}/{}",
        &command.url, &manifest.namespace, &manifest.identifier
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        command.auth.unwrap_or_else(|| "fuel".into()).parse()?,
    );

    info!(
        "\n🛑 Removing index '{}.{}' at {}",
        &manifest.namespace, &manifest.identifier, &target
    );

    let res = Client::new()
        .delete(&target)
        .headers(headers)
        .send()
        .expect("Failed to remove index.");

    if res.status() != StatusCode::OK {
        error!(
            "\n❌ {} returned a non-200 response code: {:?}",
            &target,
            res.status()
        );
        return Ok(());
    }

    let res_json = res
        .json::<Map<String, Value>>()
        .expect("Failed to read JSON response.");

    println!("\n{}", to_string_pretty(&res_json)?);

    info!(
        "\n✅ Successfully removed index '{}.{}' at {} \n",
        &manifest.namespace, &manifest.identifier, &target
    );

    Ok(())
}
