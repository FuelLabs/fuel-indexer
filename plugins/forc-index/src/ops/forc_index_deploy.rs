use crate::{
    cli::{BuildCommand, DeployCommand},
    commands::build,
    utils::{extract_manifest_fields, log::LoggerConfig, project_dir_info},
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
    time::Duration,
};
use tracing::{error, info};

pub fn init(command: DeployCommand) -> anyhow::Result<()> {
    let DeployCommand {
        url,
        manifest,
        path,
        auth,
        target,
        release,
        profile,
        verbose,
        locked,
        native,
        output_dir_root,
    } = command;

    build::exec(BuildCommand {
        manifest: manifest.clone(),
        path: path.clone(),
        target,
        release,
        profile,
        verbose,
        locked,
        native,
        output_dir_root: output_dir_root.clone(),
    })?;

    let (_root_dir, manifest_path, _index_name) =
        project_dir_info(path.as_ref(), manifest.as_ref())?;

    let mut manifest_file = fs::File::open(&manifest_path)?;
    let mut manifest_contents = String::new();
    manifest_file.read_to_string(&mut manifest_contents)?;
    let manifest: serde_yaml::Value = serde_yaml::from_str(&manifest_contents)?;

    let (namespace, identifier, graphql_schema, module_path) =
        extract_manifest_fields(manifest, output_dir_root.as_ref())?;

    let mut manifest_buff = Vec::new();
    let mut manifest_reader = BufReader::new(manifest_file);
    manifest_reader.read_to_end(&mut manifest_buff)?;

    let form = Form::new()
        .file("manifest", &manifest_path)?
        .file("schema", graphql_schema)?
        .file("wasm", module_path)?;

    let target = format!("{url}/api/index/{namespace}/{identifier}");

    info!(
        "Deploying indexer at {} to {}",
        manifest_path.display(),
        target
    );

    let mut headers = HeaderMap::new();
    if let Some(auth) = auth {
        headers.insert(AUTHORIZATION, auth.parse()?);
    }

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
        .expect("Failed to deploy indexer.");

    let status = res.status();
    let res_json = res
        .json::<Map<String, Value>>()
        .expect("Failed to read JSON response.");

    info!("\n{}", to_string_pretty(&res_json)?);
    if status != StatusCode::OK {
        error!("\n❌ {target} returned a non-200 response code: {status:?}",);

        println!("\n{}", to_string_pretty(&res_json)?);

        return Ok(());
    }

    println!("\n{}", to_string_pretty(&res_json)?);

    pb.finish_with_message("✅ Successfully deployed indexer.");

    Ok(())
}
