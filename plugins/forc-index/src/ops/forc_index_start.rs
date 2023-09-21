use crate::cli::StartCommand;
use fuel_indexer_lib::defaults;
use std::{ffi::OsStr, process::Command};
use tracing::info;

pub async fn init(command: StartCommand) -> anyhow::Result<()> {
    let StartCommand {
        manifest,
        config,
        fuel_node_host,
        fuel_node_port,
        web_api_host,
        web_api_port,
        log_level,
        run_migrations,
        metrics,
        auth_enabled,
        auth_strategy,
        jwt_secret,
        jwt_issuer,
        jwt_expiry,
        database,
        postgres_user,
        postgres_password,
        postgres_host,
        postgres_port,
        postgres_database,
        embedded_database,
        verbose,
        local_fuel_node,
        max_body_size,
        stop_idle_indexers,
        indexer_net_config,
        rate_limit,
        rate_limit_request_count,
        rate_limit_window_size,
        metering_points,
        replace_indexer,
        remove_data,
        accept_sql_queries,
        block_page_size,
    } = command;

    let mut cmd = Command::new("fuel-indexer");
    cmd.arg("run");

    if let Some(m) = &manifest {
        cmd.arg("--manifest").arg(m);
    }

    let rate_limit_window_size = rate_limit_window_size
        .map(|x| x.to_string())
        .unwrap_or(defaults::RATE_LIMIT_WINDOW_SIZE.to_string());
    let rate_limit_window_size = OsStr::new(&rate_limit_window_size);
    let rate_limit_request_count = rate_limit_request_count
        .map(|x| x.to_string())
        .unwrap_or(defaults::RATE_LIMIT_REQUEST_COUNT.to_string());
    let rate_limit_request_count = OsStr::new(&rate_limit_request_count);

    if let Some(c) = &config {
        cmd.arg("--config").arg(c);
    } else {
        // Options that have default values
        cmd.arg("--fuel-node-host").arg(&fuel_node_host);
        cmd.arg("--fuel-node-port").arg(&fuel_node_port);
        cmd.arg("--web-api-host").arg(&web_api_host);
        cmd.arg("--web-api-port").arg(&web_api_port);
        cmd.arg("--log-level").arg(&log_level);
        cmd.arg("--max-body-size")
            .arg(OsStr::new(&max_body_size.to_string()));
        cmd.arg("--rate-limit-request-count")
            .arg(rate_limit_request_count);
        cmd.arg("--rate-limit-window-size")
            .arg(rate_limit_window_size);
        cmd.arg("--metering-points")
            .arg(OsStr::new(&metering_points.to_string()));
        cmd.arg("--block-page-size")
            .arg(OsStr::new(&block_page_size.to_string()));

        // Bool options
        let options = [
            ("--embedded-database", embedded_database),
            ("--rate-limit", rate_limit),
            ("--indexer-net-config", indexer_net_config),
            ("--stop-idle-indexers", stop_idle_indexers),
            ("--replace-indexer", replace_indexer),
            ("--remove-data", remove_data),
            ("--accept-sql-queries", accept_sql_queries),
            ("--run-migrations", run_migrations),
            ("--metrics", metrics),
            ("--auth-enabled", auth_enabled),
            ("--verbose", verbose),
            ("--local-fuel-node", local_fuel_node),
        ];
        for (opt, value) in options.iter() {
            if *value {
                cmd.arg(opt);
            }
        }

        // Nullable options
        let options = [
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
                let postgres_optionals = [
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
            _ => unreachable!(
                "'postgres' is currently the only supported database option."
            ),
        }
    }

    if verbose {
        info!("{cmd:?}");
    }

    match cmd.spawn() {
        Ok(child) => {
            let pid = child.id();
            info!("✅ Successfully started the indexer service at PID {pid}");
        }
        Err(e) => panic!("❌ Failed to spawn fuel-indexer child process: {e:?}."),
    }

    Ok(())
}
