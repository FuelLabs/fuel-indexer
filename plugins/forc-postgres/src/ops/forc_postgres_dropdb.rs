use crate::{
    cli::DropDbCommand,
    utils::{default_indexer_dir, into_postgres_version, load_pg_config},
};
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
    } = command;

    let (database_dir, pgconfig) =
        load_pg_config(database_dir.as_ref(), config.as_ref(), &name)?;

    let fetch_settings = PgFetchSettings {
        version: into_postgres_version(&pgconfig.postgres_version),
        ..Default::default()
    };

    let mut pg = PgEmbed::new(pgconfig.into(), fetch_settings).await?;

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
                            fs::remove_dir_all(&database_dir)?;
                            fs::remove_file(
                                default_indexer_dir().join(pg_password_filename(&name)),
                            )?;
                            info!("\n⚠️  Please wait a few seconds before trying to re-create the same database.\n");
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
