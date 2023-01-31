use crate::{
    cli::StopDbCommand,
    utils::{into_postgres_version, load_pg_config},
};
use pg_embed::{pg_fetch::PgFetchSettings, postgres::PgEmbed};
use tracing::info;

pub async fn init(command: StopDbCommand) -> anyhow::Result<()> {
    let StopDbCommand {
        name,
        database_dir,
        config,
    } = command;

    let (_database_dir, pgconfig) =
        load_pg_config(database_dir.as_ref(), config.as_ref(), &name)?;

    let fetch_settings = PgFetchSettings {
        version: into_postgres_version(&pgconfig.postgres_version),
        ..Default::default()
    };

    let mut pg = PgEmbed::new(pgconfig.into(), fetch_settings).await?;

    let pg_db_uri = pg.full_db_uri(&name);

    info!("\nStopping database at '{pg_db_uri}'.\n");

    match pg.database_exists(&name).await {
        Ok(exists) => {
            if exists {
                info!("\nStopping database at '{pg_db_uri}'.\n");
                match pg.stop_db().await {
                    Ok(_) => {
                        info!("✅ Successfully stopped database at '{pg_db_uri}'.");
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
