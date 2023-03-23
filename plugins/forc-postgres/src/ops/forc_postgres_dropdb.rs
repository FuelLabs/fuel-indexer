use crate::{cli::DropDbCommand, pg::PgEmbedConfig, utils::default_indexer_dir};
use pg_embed::{pg_fetch::PgFetchSettings, postgres::PgEmbed};
use std::fs;
use tracing::info;

fn pg_password_filename(name: &str) -> String {
    format!("{name}.pwfile")
}

pub async fn init(command: DropDbCommand) -> anyhow::Result<()> {
    let DropDbCommand {
        name,
        database_dir,
        config,
        remove_persisted,
        ..
    } = command;

    let pg_config =
        PgEmbedConfig::from_file(database_dir.as_ref(), config.as_ref(), &name)?;

    let fetch_settings = PgFetchSettings {
        version: pg_config.postgres_version.clone().into(),
        ..Default::default()
    };

    let mut pg = PgEmbed::new(pg_config.clone().into(), fetch_settings).await?;

    let pg_db_uri = pg.full_db_uri(&name);

    info!("\nStarting PostgreSQL.\n");
    pg.start_db().await?;

    match pg.database_exists(&name).await {
        Ok(exists) => {
            if exists {
                info!("\nDropping database at '{pg_db_uri}'.\n");
                match pg.drop_database(&name).await {
                    Ok(_) => {
                        info!("✅ Successfully dropped database at '{pg_db_uri}'.");
                        if remove_persisted {
                            fs::remove_dir_all(pg_config.database_dir.unwrap())?;
                            fs::remove_file(
                                default_indexer_dir().join(pg_password_filename(&name)),
                            )?;
                            info!(
                                r#"
⚠️  Please wait at least 30 seconds before trying to re-create the same database.
The `drop` operation might still be running on the specified port and needs time to finish.
"#
                            );
                        }
                    }
                    Err(e) => {
                        anyhow::bail!(
                            "❌ Failed to drop database at '{pg_db_uri}': {e}."
                        );
                    }
                }
            } else {
                anyhow::bail!("❌ Database at '{pg_db_uri}' does not exist.");
            }
        }
        Err(e) => {
            anyhow::bail!("❌ Error determining database's existence: {e}.\n⚠️  Did you stop your the database at '{pg_db_uri}' before trying to drop it?");
        }
    }

    Ok(())
}
