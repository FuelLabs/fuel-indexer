use crate::cli::StartCommand;
use forc_postgres::cli::CreateDbCommand;
use fuel_indexer_lib::defaults;
use std::process::Command;
use tracing::info;

pub async fn init(command: StartCommand) -> anyhow::Result<()> {
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
        embedded_database,
        auth_enabled,
        auth_strategy,
        jwt_secret,
        jwt_issuer,
        jwt_expiry,
        verbose_logging,
        verbose_db_logging,
        ..
    } = command;

    if embedded_database {
        let name = postgres_database
            .clone()
            .unwrap_or(defaults::POSTGRES_DATABASE.to_string());
        let password = postgres_password
            .clone()
            .unwrap_or(defaults::POSTGRES_PASSWORD.to_string());
        let user = postgres_user
            .clone()
            .unwrap_or(defaults::POSTGRES_USER.to_string());

        let port = postgres_port
            .clone()
            .unwrap_or(defaults::POSTGRES_PORT.to_string());

        let create_db_cmd = CreateDbCommand {
            name,
            password,
            user,
            port,
            config: config.clone(),
            start: true,
            ..Default::default()
        };

        forc_postgres::commands::create::exec(Box::new(create_db_cmd)).await?;
    }

    let mut cmd = Command::new("fuel-indexer");
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
        cmd.arg("--verbose-db-logging").arg(&verbose_db_logging);

        // Bool options
        let options = vec![
            ("--run-migrations", run_migrations),
            ("--metrics", metrics),
            ("--auth-enabled", auth_enabled),
            ("--verbose-logging", verbose_logging),
        ];
        for (opt, value) in options.iter() {
            if *value {
                cmd.arg(opt);
            }
        }

        // Nullable options
        let options = vec![
            ("--auth-strategy", auth_strategy),
            ("--jwt-secret", jwt_secret),
            ("--jwt-issuer", jwt_issuer),
            ("--jwt-expiry", jwt_expiry.map(|x| x.to_string())),
        ];
        for (opt, value) in options.iter() {
            if let Some(value) = value {
                cmd.arg(opt).arg(value);
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

    if verbose_logging {
        info!("{cmd:?}");
    }

    if let Ok(child) = cmd.spawn() {
        info!(
            "\n✅ Successfully started the indexer service at PID {}.",
            child.id()
        );
    } else {
        anyhow::bail!("❌ Failed to spawn fuel-indexer child process.");
    }

    Ok(())
}
