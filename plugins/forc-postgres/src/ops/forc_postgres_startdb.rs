use crate::{
    cli::StartDbCommand,
    utils::{into_postgres_version, load_pg_config},
};
use pg_embed::{pg_fetch::PgFetchSettings, postgres::PgEmbed};
use tracing::info;

pub fn run_forever() {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

pub async fn init(command: StartDbCommand) -> anyhow::Result<()> {
    let StartDbCommand {
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

    info!("\nStarting PostgreSQL.\n");
    pg.start_db().await?;

    let pg_db_uri = pg.full_db_uri(&name);

    match pg.database_exists(&name).await {
        Ok(exists) => {
            if exists {
                info!("✅ Successfully started database at '{pg_db_uri}'.");
                run_forever();
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
