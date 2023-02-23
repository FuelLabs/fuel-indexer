use crate::ops::forc_index_start;
use crate::{
    cli::{RevertCommand, StartCommand},
    utils::{defaults, project_dir_info},
};
use fuel_indexer_lib::{
    manifest::Manifest,
    defaults as indexer_defaults,
};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, AUTHORIZATION},
};
use std::time::Duration;
use tracing::info;

pub fn init(command: RevertCommand) -> anyhow::Result<()> {
    let RevertCommand { path, manifest, .. } = command;

    let (_root_dir, manifest_path, _index_name) =
        project_dir_info(path.as_ref(), manifest.as_ref())?;

    println!("manifest_path: {:?}", manifest_path);

    let manifest: Manifest = Manifest::from_file(manifest_path.as_path())?;

    println!("manifest: {:?}", manifest);

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
        "\n⬅️  Reverting index '{}.{}' at {}",
        &manifest.namespace, &manifest.identifier, &target
    );

    let res = Client::new()
        .get(&target)
        .headers(headers)
        .send()
        .expect("Failed to fetch recent index.");

    println!("res: {:?}", res);

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

    pb.finish_with_message("✅ Successfully deployed indexer.");

    let start_command = generate_start_command();
    forc_index_start::init(start_command)?;
    Ok(())
}

fn generate_start_command() -> StartCommand {
    StartCommand {
        log_level: "info".to_string(),
        config: None,
        manifest: None,
        fuel_node_host: indexer_defaults::FUEL_NODE_HOST.to_string(),
        fuel_node_port: indexer_defaults::FUEL_NODE_PORT.to_string(),
        graphql_api_host: defaults::GRAPHQL_API_HOST.to_string(),
        graphql_api_port: defaults::GRAPHQL_API_PORT.to_string(),
        database: indexer_defaults::DATABASE.to_string(),
        postgres_user: Some(indexer_defaults::POSTGRES_USER.to_string()),
        postgres_database: Some(indexer_defaults::POSTGRES_DATABASE.to_string()),
        postgres_password: Some(indexer_defaults::POSTGRES_PASSWORD.to_string()),
        postgres_host: Some(indexer_defaults::POSTGRES_HOST.to_string()),
        postgres_port: Some(indexer_defaults::POSTGRES_PORT.to_string()),
        max_body: indexer_defaults::MAX_BODY.to_string(),
        run_migrations: true,
        metrics: false,
        stop_idle_indexers: false,
    }
}
