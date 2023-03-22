use crate::{cli::StartDbCommand, pg::PgEmbedConfig};
use pg_embed::{pg_fetch::PgFetchSettings, postgres::PgEmbed};
use std::mem::ManuallyDrop;
use tracing::info;

pub async fn init(command: StartDbCommand) -> anyhow::Result<()> {
    let StartDbCommand {
        name,
        database_dir,
        config,
        verbose,
        ..
    } = command;

    let pg_config =
        PgEmbedConfig::from_file(database_dir.as_ref(), config.as_ref(), &name)?;

    let version = pg_config.postgres_version.clone();

    let fetch_settings = PgFetchSettings {
        version: version.clone().into(),
        ..Default::default()
    };

    // Disabling Drop trait behavior as PgEmbed shuts down when going out of scope
    let mut pg =
        ManuallyDrop::new(PgEmbed::new(pg_config.clone().into(), fetch_settings).await?);

    info!("\nStarting PostgreSQL...\n");
    pg.start_db().await?;

    let pg_db_uri = pg.full_db_uri(&name);

    match pg.database_exists(&name).await {
        Ok(exists) => {
            if exists {
                if verbose {
                    info!("\n✅ Successfully started database at '{pg_db_uri}'.");
                } else {
                    info!("\n✅ Successfully started database.");
                }
            } else {
                anyhow::bail!("❌ Database at '{pg_db_uri}' does not exist.");
            }
        }
        Err(e) => {
            anyhow::bail!("❌ Error determining database's existence: {e}.");
        }
    }

    Ok(())
}
