use crate::cli::CheckCommand;
use reqwest::{blocking::Client, StatusCode};
use serde_json::{value::Value, Map};
use std::process::Command;
use tracing::error;

fn center_align(s: &str, n: usize) -> String {
    format!("{: ^width$}", s, width = n)
}

fn format_exec_msg(exec_name: &str, path: Option<String>) -> String {
    if path.is_some() {
        center_align(
            &format!("Found '{}' located at '{}'", exec_name, path.unwrap()),
            76,
        )
    } else {
        center_align(&format!("Could not located '{}'", exec_name), 76)
    }
}

fn find_executable(exec_name: &str) -> (String, Option<String>) {
    match Command::new("which").arg(exec_name).output() {
        Ok(o) => {
            let path = String::from_utf8_lossy(&o.stdout)
                .strip_suffix('\n')
                .map(|x| x.to_string());
            (center_align("✅", 5), path)
        }
        Err(e) => {
            error!("Could not locate {}: {}", exec_name, e);
            (center_align("❌", 5), None)
        }
    }
}

fn find_executable_with_fallback(
    exec_name: &str,
    fallback_exec_name: &str,
) -> (String, Option<String>) {
    let (emoji, path) = find_executable(exec_name);
    if path.is_some() {
        (emoji, path)
    } else {
        let (emoji, path) = find_executable(fallback_exec_name);
        (emoji, path)
    }
}

fn find_indexer_service_info(cmd: &CheckCommand) -> (String, String) {
    let (emoji, msg) = match Command::new("lsof")
        .arg(&format!("-ti:{}", cmd.grpahql_api_port))
        .output()
    {
        Ok(o) => {
            let (emoji, msg) = match String::from_utf8_lossy(&o.stdout)
            .to_string()
            .strip_suffix('\n')  {
            Some(pid) => (
                center_align("✅", 5),
                center_align(&format!(
                    "Local fuel-indexer service found: PID({}) | Port({})",
                    &pid, &cmd.grpahql_api_port
                ), 76)
            ),
            None => (
                center_align("⛔️", 5),
                center_align(&format!(
                    "Failed to detect a locally running fuel-indexer service at Port({}).",
                    cmd.grpahql_api_port
                ), 76))
        };

            (emoji, msg)
        }
        Err(e) => {
            error!("Could not find info for fuel-indexer service: {}", e);
            (
                center_align("⚠️", 5),
                center_align(
                    &format!(
                "Failed to detect a locally running fuel-indexer service at port: {}.",
                cmd.grpahql_api_port
            ),
                    76,
                ),
            )
        }
    };

    (emoji, msg)
}

pub fn init(command: CheckCommand) -> anyhow::Result<()> {
    let target = format!("{}/api/health", command.url);

    let psql = "psql";
    let sqlite = "sqlite";
    let sqlite3 = "sqlite3";
    let fuel_indexer = "fuel-indexer";

    match Client::new().get(&target).send() {
        Ok(res) => {
            if res.status() != StatusCode::OK {
                error!(
                    "\n❌ {} returned a non-200 response code: {:?}",
                    &target,
                    res.status()
                );
                return Ok(());
            }

            let _res_json = res
                .json::<Map<String, Value>>()
                .expect("Failed to read JSON response.");
        }
        Err(e) => {
            error!("Could not connect to Indexer service: {}", e);
        }
    }

    let (binary_emoji, binary_path) = find_executable(fuel_indexer);
    let binary_msg = format_exec_msg(fuel_indexer, binary_path);
    let (psql_emoji, psql_path) = find_executable(psql);
    let psql_msg = format_exec_msg(psql, psql_path);
    let (sqlite_emoji, sqlite_path) = find_executable_with_fallback(sqlite, sqlite3);
    let sqlite_msg = format_exec_msg(sqlite, sqlite_path);
    let (service_emoji, service_msg) = find_indexer_service_info(&command);

    let details_header = center_align("Details", 76);
    let check_header = center_align("Component", 30);
    let status_headers = center_align("Status", 7);
    let binary_header = center_align("fuel-indexer binary", 30);
    let service_header = center_align("fuel-indexer service", 30);
    let psql_header = center_align("postgres", 30);
    let sqlite_header = center_align("sqlite", 30);

    // TODO: Simplify this by just padding/justifying the strings (>'.')>
    let stdout = format!(
        r#"   
+----------+----------------------------------+----------------------------------------------------------------------------+
|  {status_headers} |  {check_header}  |{details_header}|
+----------+----------------------------------+----------------------------------------------------------------------------+
|  {binary_emoji}  |  {binary_header}  |{binary_msg}|
+----------+----------------------------------+----------------------------------------------------------------------------+
|  {service_emoji}  | {service_header}   |{service_msg}|
+----------+----------------------------------+----------------------------------------------------------------------------+
|  {psql_emoji}  | {psql_header}   |{psql_msg}|
+----------+----------------------------------+----------------------------------------------------------------------------+
|  {sqlite_emoji}  | {sqlite_header}   |{sqlite_msg}|
+----------+----------------------------------+----------------------------------------------------------------------------+
"#
    );

    println!("{}", stdout);

    Ok(())
}
