use crate::cli::StartCommand;
use std::process::Command;
use tracing::info;

pub fn init(command: StartCommand) -> anyhow::Result<()> {
    let StartCommand {
        log_level,
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
        manifest,
        ..
    } = command;

    let stdout = Command::new("which")
        .arg("fuel-indexer")
        .output()
        .expect("❌ Failed to locate fuel-indexer binary.")
        .stdout;

    let exec = String::from_utf8_lossy(&stdout)
        .strip_suffix('\n')
        .expect("Failed to detect fuel-indexer binary in $PATH.")
        .to_string();

    let mut cmd = Command::new(&exec);
    cmd.arg("run");

    if let Some(m) = &manifest {
        cmd.arg("--manifest").arg(m);
    }

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

    let _proc = cmd
        .spawn()
        .expect("❌ Failed to spawn fuel-indexer child process.");

    info!("\n✅ Successfully started the indexer service.");

    Ok(())
}
