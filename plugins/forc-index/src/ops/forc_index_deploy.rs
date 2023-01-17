use crate::{
    cli::{BuildCommand, DeployCommand},
    commands::build,
    utils::extract_manifest_fields,
};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{
    blocking::{multipart::Form, Client},
    header::{HeaderMap, AUTHORIZATION},
    StatusCode,
};
use serde_json::{to_string_pretty, value::Value, Map};
use std::{
    fs,
    io::{BufReader, Read},
    path::Path,
    time::Duration,
};
use tracing::{error, info};

pub fn init(command: DeployCommand) -> anyhow::Result<()> {
    let DeployCommand {
        host,
        manifest,
        auth,
        target,
        release,
        profile,
        verbose,
        locked,
        native,
    } = command;

    build::exec(BuildCommand {
        manifest: manifest.clone(),
        target,
        release,
        profile,
        verbose,
        locked,
        native,
    })?;

    let manifest_path = Path::new(&manifest);
    let mut manifest_file = fs::File::open(manifest_path)?;
    let mut manifest_contents = String::new();
    manifest_file.read_to_string(&mut manifest_contents)?;
    let manifest: serde_yaml::Value = serde_yaml::from_str(&manifest_contents)?;

    let (namespace, identifier, graphql_schema, module_path) =
        extract_manifest_fields(manifest)?;

    let mut manifest_buff = Vec::new();
    let mut manifest_reader = BufReader::new(manifest_file);
    manifest_reader.read_to_end(&mut manifest_buff)?;

    let form = Form::new()
        .file("manifest", manifest_path)?
        .file("schema", Path::new(&graphql_schema))?
        .file("wasm", Path::new(&module_path))?;

    let target = format!("{}/api/index/{}/{}", &host, &namespace, &identifier);

    info!(
        "Deploying index at {} to {}",
        manifest_path.display(),
        target
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        auth.unwrap_or_else(|| "fuel".into()).parse()?,
    );

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ]),
    );
    pb.set_message("🚀 Deploying...");

    let res = Client::new()
        .post(&target)
        .multipart(form)
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

    pb.finish_with_message("✅ Successfully deployed index.");

    Ok(())
}
