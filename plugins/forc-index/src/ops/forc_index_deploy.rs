use crate::{
    cli::{BuildCommand, DeployCommand},
    commands::build,
    utils::project_dir_info,
};
use fuel_indexer_lib::manifest::Manifest;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{
    blocking::{multipart::Form, Client},
    header::{HeaderMap, AUTHORIZATION, CONNECTION},
    StatusCode,
};
use serde_json::{to_string_pretty, value::Value, Map};
use std::{path::Path, time::Duration};
use tracing::{error, info};

const STEADY_TICK_INTERVAL: u64 = 120;
const TCP_TIMEOUT: u64 = 3;

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
        stop_previous,
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
            target_dir: target_dir.clone(),
        })?;
    }

    let (_root_dir, manifest_path, _index_name) =
        project_dir_info(path.as_ref(), manifest.as_ref())?;

    let manifest = Manifest::from_file(&manifest_path)?;

    let Manifest {
        mut graphql_schema,
        namespace,
        identifier,
        mut module,
        ..
    } = manifest;

    if path.is_some() {
        if let Some(t) = target_dir {
            graphql_schema = Path::new(&t)
                .join(graphql_schema)
                .to_str()
                .unwrap()
                .to_string();

            module = Path::new(&t).join(module).into();
        } else {
            anyhow::bail!("--target-dir must be specified when --path is specified.");
        }
    }

    let form = Form::new()
        .file("manifest", &manifest_path)?
        .file("schema", graphql_schema)?
        .file("wasm", module.to_string())?;

    let target =
        format!("{url}/api/index/{namespace}/{identifier}?stop_previous={stop_previous}");

    if verbose {
        info!(
            "Deploying indexer at {} to {}.",
            manifest_path.display(),
            target
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
        .expect("Failed to deploy indexer.");

    let status = res.status();
    let res_json = res
        .json::<Map<String, Value>>()
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
