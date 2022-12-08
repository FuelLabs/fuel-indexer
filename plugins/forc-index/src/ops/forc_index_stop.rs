use crate::cli::StopCommand;
use crate::utils::extract_manifest_fields;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, AUTHORIZATION},
    StatusCode,
};
use serde_json::{to_string_pretty, value::Value, Map};
use std::fs;
use std::io::Read;
use tracing::{error, info};

pub fn init(command: StopCommand) -> anyhow::Result<()> {
    let mut manifest_file = fs::File::open(&command.manifest).unwrap_or_else(|_| {
        panic!(
            "Index manifest file at '{}' does not exist",
            command.manifest.display()
        )
    });

    let mut manifest_contents = String::new();
    manifest_file.read_to_string(&mut manifest_contents)?;
    let manifest: serde_yaml::Value = serde_yaml::from_str(&manifest_contents)?;

    let (namespace, identifier, _, _) = extract_manifest_fields(manifest)?;

    let target = format!("{}/api/index/{}/{}", &command.url, &namespace, &identifier);

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        command.auth.unwrap_or_else(|| "fuel".into()).parse()?,
    );

    info!("\n🛑 Stopping index at {}", &target);

    let res = Client::new()
        .delete(&target)
        .headers(headers)
        .send()
        .expect("Failed to deploy index.");

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

    info!("\n✅ Successfully stopped index at {} \n", &target);

    Ok(())
}
