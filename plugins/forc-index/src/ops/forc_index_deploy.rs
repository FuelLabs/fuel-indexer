use crate::{
    cli::{BuildCommand, DeployCommand},
    commands::build,
    utils::project_dir_info,
};
use fuel_indexer_lib::manifest::Manifest;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{
    blocking::{multipart::Form, Client},
    header::{HeaderMap, AUTHORIZATION},
    StatusCode,
};
use serde_json::{to_string_pretty, value::Value, Map};
use std::time::Duration;
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
        locked,
        native,
        target_dir,
        verbose,
        skip_build,
    } = command;

    if !skip_build {
        build::exec(BuildCommand {
            manifest: manifest.clone(),
            path: path.clone(),
            target,
            release,
            profile,
            verbose,
            locked,
            native,
            target_dir,
        })?;
    }

    let (_root_dir, manifest_path, _index_name) =
        project_dir_info(path.as_ref(), manifest.as_ref())?;

    let manifest = Manifest::from_file(&manifest_path)?;

    let Manifest {
        graphql_schema,
        namespace,
        identifier,
        module,
        ..
    } = manifest;

    let form = Form::new()
        .file("manifest", &manifest_path)?
        .file("schema", graphql_schema)?
        .file("wasm", module.path())?;

    let target = format!("{url}/api/index/{namespace}/{identifier}");

    if verbose {
        info!(
            "Deploying indexer at {} to {}",
            manifest_path.display(),
            target
        );
    } else {
        info!("Deploying indexer");
    }

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

    if status != StatusCode::OK {
        error!("\n❌ {target} returned a non-200 response code: {status:?}",);

        println!("\n{}", to_string_pretty(&res_json)?);

        return Ok(());
    }

    if verbose {
        info!("\n{}", to_string_pretty(&res_json)?);
    }

    pb.finish_with_message("✅ Successfully deployed indexer.");

    Ok(())
}
