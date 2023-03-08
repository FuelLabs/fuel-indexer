use crate::{
    cli::{CreateDbCommand, StartDbCommand},
    commands::start,
    pg::{PgEmbedConfig, PostgresVersion},
    utils::{db_config_file_name, default_indexer_dir},
};
use anyhow::Result;
use fuel_indexer_lib::{
    config::{DatabaseConfig, IndexerConfig},
    defaults,
};
use indicatif::{ProgressBar, ProgressStyle};
use pg_embed::{pg_fetch::PgFetchSettings, postgres::PgEmbed};
use std::{fs::File, io::Write, path::PathBuf, time::Duration};
use tracing::info;

fn save_pgembed_config(config: PgEmbedConfig, path: Option<&PathBuf>) -> Result<()> {
    if let Some(path) = path {
        let filename = db_config_file_name(&config.name);
        let path = path.join(filename);
        info!("\nWriting PgEmbedConfig to {path:?}");
        let mut file = File::create(path)?;
        file.write_all(serde_json::to_string(&config)?.as_bytes())?;
    }

    Ok(())
}

impl From<CreateDbCommand> for PgEmbedConfig {
    fn from(val: CreateDbCommand) -> Self {
        let CreateDbCommand {
            name,
            user,
            password,
            port,
            database_dir,
            auth_method,
            persistent,
            timeout,
            migration_dir,
            postgres_version,
            ..
        } = val;
        Self {
            name,
            user,
            password,
            port: port.parse::<u16>().expect("Invalid port."),
            database_dir,
            auth_method,
            persistent,
            timeout,
            migration_dir,
            postgres_version,
        }
    }
}

impl From<IndexerConfig> for PgEmbedConfig {
    fn from(val: IndexerConfig) -> Self {
        let IndexerConfig { database, .. } = val;

        match database {
            DatabaseConfig::Postgres {
                user,
                password,
                port,
                database: name,
                ..
            } => Self {
                name,
                user,
                password,
                port: port.parse::<u16>().expect("Invalid port."),
                database_dir: Some(default_indexer_dir()),
                auth_method: "plain".to_string(),
                persistent: true,
                timeout: None,
                migration_dir: None,
                postgres_version: PostgresVersion::V14,
            },
        }
    }
}

pub async fn init(command: CreateDbCommand) -> anyhow::Result<()> {
    let CreateDbCommand {
        name,
        database_dir,
        migration_dir,
        start,
        config,
        ..
    } = command.clone();

    let pg_config: PgEmbedConfig = if config.is_some() {
        IndexerConfig::from_file(&config.clone().unwrap())?.into()
    } else {
        command.into()
    };

    let fetch_settings = PgFetchSettings {
        version: pg_config.postgres_version.clone().into(),
        ..Default::default()
    };

    let mut pg = PgEmbed::new(pg_config.clone().into(), fetch_settings).await?;

    let pg_db_uri = pg.full_db_uri(&name);

    info!("📦 Downloading, unpacking, and bootstrapping database...\n");
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
    pb.set_message("⏱  Setting up database...\n");

    pg.setup().await?;

    pg.start_db().await?;

    info!("\n💡 Creating database at '{pg_db_uri}'.");

    if let Err(e) = pg.create_database(&name).await {
        if name == defaults::POSTGRES_DATABASE {
            info!(
                "\nDefault database {} already exists.\n",
                defaults::POSTGRES_DATABASE
            );
        } else {
            anyhow::bail!(e);
        }
    }

    if migration_dir.is_some() {
        pg.migrate(&name).await?;
    }

    save_pgembed_config(pg_config, database_dir.as_ref())?;

    pb.finish();

    info!("\n✅ Successfully created database at '{pg_db_uri}'.");

    if start {
        // Allow for start command to fully manage PgEmbed object
        pg.stop_db().await?;

        start::exec(StartDbCommand {
            name,
            database_dir: Some(database_dir.unwrap()),
            config,
        })
        .await?;
    }

    Ok(())
}
