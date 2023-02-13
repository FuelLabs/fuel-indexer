use crate::cli::StartCommand;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

pub fn init(command: StartCommand) -> anyhow::Result<()> {
    let StartCommand {
        log_level,
        bin,
        background,
        config,
        fuel_node_host,
        fuel_node_port,
        graphql_api_host,
        graphql_api_port,
        database,
        postgres_user,
        postgres_password,
        postgres_database,
        postgres_host,
        postgres_port,
        run_migrations,
        metrics,
        ..
    } = command;

    // If the user has a binary path they'd prefer to use, they can specify
    // it, else just use whichever indexer is in the path - whether that be
    // in fuelup or some other means.
    let mut cmd = Command::new(&bin.unwrap_or_else(|| {
        PathBuf::from(
            String::from_utf8_lossy(
                &Command::new("which")
                    .arg("fuel-indexer")
                    .output()
                    .expect("❌ Failed to locate fuel-indexer binary.")
                    .stdout,
            )
            .strip_suffix('\n')
            .expect("Failed to detect fuel-indexer binary in $PATH."),
        )
    }));
    cmd.arg("run");

    if let Some(c) = &config {
        cmd.arg("--config").arg(c);
    } else {
        // Options that have default values
        cmd.arg("--fuel-node-host").arg(&fuel_node_host);
        cmd.arg("--fuel-node-port").arg(&fuel_node_port);
        cmd.arg("--graphql-api-host").arg(&graphql_api_host);
        cmd.arg("--graphql-api-port").arg(&graphql_api_port);
        cmd.arg("--log-level").arg(&log_level);

        // Bool options
        let options = vec![("--run-migrations", run_migrations), ("--metrics", metrics)];
        for (opt, value) in options.iter() {
            if *value {
                cmd.arg(opt).arg("true");
            }
        }

        match database.as_ref() {
            "postgres" => {
                // Postgres optional values
                let postgres_optionals = vec![
                    ("--postgres-user", postgres_user),
                    ("--postgres-password", postgres_password),
                    ("--postgres-host", postgres_host),
                    ("--postgres-port", postgres_port),
                    ("--postgres-database", postgres_database),
                ];

                for (flag, value) in postgres_optionals.iter() {
                    if let Some(v) = value {
                        cmd.arg(flag).arg(v);
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    let mut proc = cmd
        .spawn()
        .expect("❌ Failed to spawn fuel-indexer child process.");

    // Starting the service in the background allows the user to
    // go and and continue interacting with the service (e.g., forc index deploy)
    // without having to switch terminals
    if !background {
        let ecode = proc
            .wait()
            .expect("❌ Failed to wait on fuel-indexer process.");

        if !ecode.success() {
            anyhow::bail!(
                "❌ fuel-indexer process did not exit successfully (Code: {:?}",
                ecode
            );
        }
    }

    info!("\n✅ Successfully started the indexer service.");

    Ok(())
}
