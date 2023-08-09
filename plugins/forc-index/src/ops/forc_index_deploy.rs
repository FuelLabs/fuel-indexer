use crate::{
    cli::{BuildCommand, DeployCommand},
    commands::build,
    utils::{file_part, project_dir_info},
};
use fuel_indexer_lib::manifest::Manifest;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{
    header::{HeaderMap, AUTHORIZATION, CONNECTION},
    multipart::Form,
    Client, StatusCode,
};
use serde_json::{to_string_pretty, value::Value, Map};
use std::{path::Path, time::Duration};
use tracing::{error, info};

const STEADY_TICK_INTERVAL: u64 = 120;
const TCP_TIMEOUT: u64 = 3;

pub async fn init(command: DeployCommand) -> anyhow::Result<()> {
    let DeployCommand {
        url,
        manifest,
        path,
        auth,
        debug,
        locked,
        native,
        verbose,
        skip_build,
    } = command;

    if !skip_build {
        build::exec(BuildCommand {
            manifest: manifest.clone(),
            path: path.clone(),
            debug,
            verbose,
            locked,
            native,
        })?;
    }

    let (_root_dir, manifest_path, _index_name) =
        project_dir_info(path.as_ref(), manifest.as_ref())?;

    let manifest = Manifest::from_file(&manifest_path)?;

    let current_dir = std::env::current_dir()?;

    let path = path.unwrap_or(current_dir);

    let workspace_root =
        crate::ops::utils::cargo_workspace_root_dir(path.as_path()).unwrap();

    let manifest_schema_file = Path::new(&workspace_root)
        .join(manifest.graphql_schema())
        .to_str()
        .unwrap()
        .to_string();

    let manifest_modulue_file = workspace_root.join(manifest.module());

    let form = Form::new()
        .part("manifest", file_part(&manifest_path).await?)
        .part("schema", file_part(manifest_schema_file).await?)
        .part("wasm", file_part(manifest_modulue_file).await?);

    let target = format!(
        "{url}/api/index/{}/{}",
        manifest.namespace(),
        manifest.identifier()
    );

    if verbose {
        info!(
            "Deploying indexer at {} to {target}.",
            manifest_path.display()
        );
    } else {
        info!("Deploying indexer...");
    }

    let mut headers = HeaderMap::new();
    headers.insert(CONNECTION, "keep-alive".parse()?);
    if let Some(auth) = auth {
        headers.insert(AUTHORIZATION, auth.parse()?);
    }

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(STEADY_TICK_INTERVAL));
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

    let client = Client::builder()
        .tcp_keepalive(Duration::from_secs(TCP_TIMEOUT))
        .connection_verbose(verbose)
        .build()?;

    let res = client
        .post(&target)
        .multipart(form)
        .headers(headers)
        .send()
        .await
        .expect("Failed to deploy indexer.");

    let status = res.status();
    let res_json = res
        .json::<Map<String, Value>>()
        .await
        .expect("Failed to read JSON response.");

    if status != StatusCode::OK {
        if verbose {
            error!("\n❌ {target} returned a non-200 response code: {status:?}",);

            info!("\n{}", to_string_pretty(&res_json)?);
        } else {
            info!("\n{}", to_string_pretty(&res_json)?);
        }

        return Ok(());
    }

    if verbose {
        info!("\n{}", to_string_pretty(&res_json)?);
    }

    pb.finish_with_message("✅ Successfully deployed indexer.");

    Ok(())
}
