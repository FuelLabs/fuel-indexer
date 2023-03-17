use crate::{cli::StopDbCommand, pg::PgEmbedConfig};
use pg_embed::{pg_fetch::PgFetchSettings, postgres::PgEmbed};
use tracing::info;

pub async fn init(command: StopDbCommand) -> anyhow::Result<()> {
    let StopDbCommand {
        name,
        database_dir,
        config,
        ..
    } = command;

    let pg_config =
        PgEmbedConfig::from_file(database_dir.as_ref(), config.as_ref(), &name)?;

    let fetch_settings = PgFetchSettings {
        version: pg_config.postgres_version.clone().into(),
        ..Default::default()
    };

    let mut pg = PgEmbed::new(pg_config.into(), fetch_settings).await?;

    let pg_db_uri = pg.full_db_uri(&name);

    match pg.database_exists(&name).await {
        Ok(exists) => {
            if exists {
                info!("\nStopping database at '{pg_db_uri}'.\n");
                match pg.stop_db().await {
                    Ok(_) => {
                        println!("✅ Successfully stopped database at '{pg_db_uri}'.");
                    }
                    Err(e) => {
                        anyhow::bail!(
                            "❌ Failed to stop database at '{pg_db_uri}': {e}."
                        );
                    }
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
