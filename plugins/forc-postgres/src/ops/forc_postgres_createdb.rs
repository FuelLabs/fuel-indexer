use crate::{
    cli::{CreateDbCommand, StartDbCommand},
    commands::start,
    utils::{
        database_dir_or_default, db_config_file_name, into_postgres_version,
        PgEmbedConfig,
    },
};
use anyhow::Result;
use fuel_indexer_lib::config::{DatabaseConfig, IndexerConfig};
use indicatif::{ProgressBar, ProgressStyle};
use pg_embed::{pg_fetch::PgFetchSettings, postgres::PgEmbed};
use std::{fs::File, io::Write, path::Path, time::Duration};
use tracing::info;

pub const DEFAULT_DATABASE: &str = "postgres";

fn save_pgembed_config(config: PgEmbedConfig, path: &Path) -> Result<()> {
    let filename = db_config_file_name(&config.name);
    let path = path.join(filename);
    info!("\nWriting PgEmbedConfig to {path:?}");
    let mut file = File::create(path)?;
    file.write_all(serde_json::to_string(&config)?.as_bytes())?;
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
                database_dir: None,
                auth_method: "plain".to_string(),
                persistent: false,
                timeout: None,
                migration_dir: None,
                postgres_version: "v14".to_string(),
            },
        }
    }
}

pub async fn init(command: CreateDbCommand) -> anyhow::Result<()> {
    let CreateDbCommand {
        name,
        database_dir,
        migration_dir,
        postgres_version,
        start,
        config,
        ..
    } = command.clone();

    let database_dir = database_dir_or_default(database_dir.as_ref(), &name);
    let pgconfig: PgEmbedConfig = command.into();

    let fetch_settings = PgFetchSettings {
        version: into_postgres_version(&postgres_version),
        ..Default::default()
    };

    let mut pg = PgEmbed::new(pgconfig.clone().into(), fetch_settings).await?;

    let pg_db_uri = pg.full_db_uri(&name);

    info!("Downloading, unpacking, and bootstrapping database.");
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
    pb.set_message("⏱  Setting up database...");

    pg.setup().await?;

    info!("🎬 Starting database at '{pg_db_uri}'.");

    pg.start_db().await?;

    info!("💡 Creating database at '{pg_db_uri}'.");

    match pg.create_database(&name).await {
        Ok(_) => {}
        Err(e) => {
            if name == DEFAULT_DATABASE {
                info!("\nDefault database {DEFAULT_DATABASE} already exists.\n");
            } else {
                anyhow::bail!(e);
            }
        }
    }

    if migration_dir.is_some() {
        pg.migrate(&name).await?;
    }

    save_pgembed_config(pgconfig, &database_dir)?;

    pb.finish();

    info!("✅ Successfully created database at '{pg_db_uri}'.");

    if start {
        start::exec(StartDbCommand {
            name,
            database_dir: Some(database_dir),
            config,
        })
        .await?;
    }

    Ok(())
}
