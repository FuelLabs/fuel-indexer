use crate::{
    cli::StartDbCommand,
    pg::{get_pgembed_home_dir, PgEmbedConfig},
};
use pg_embed::{pg_fetch::PgFetchSettings, postgres::PgEmbed};
use std::process::Command;
use tracing::info;

pub async fn init(command: StartDbCommand) -> anyhow::Result<()> {
    let StartDbCommand {
        name,
        database_dir,
        config,
    } = command;

    let pg_config =
        PgEmbedConfig::from_file(database_dir.as_ref(), config.as_ref(), &name)?;

    let version = pg_config.postgres_version.clone();

    let fetch_settings = PgFetchSettings {
        version: version.clone().into(),
        ..Default::default()
    };

    let mut pg = PgEmbed::new(pg_config.clone().into(), fetch_settings).await?;

    info!("\nStarting PostgreSQL.\n");
    pg.start_db().await?;

    let pg_db_uri = pg.full_db_uri(&name);

    match pg.database_exists(&name).await {
        Ok(exists) => {
            if exists {
                info!("\n✅ Successfully started database at '{pg_db_uri}'.");

                let executable = get_pgembed_home_dir(version);

                if let Err(e) = Command::new(executable)
                    .arg("-D")
                    .arg(&pg_config.database_dir.unwrap())
                    .arg("-l")
                    .arg("logfile")
                    .arg("start")
                    .spawn()
                {
                    anyhow::bail!("❌ Failed to invoke pg_ctl: {e}.");
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
