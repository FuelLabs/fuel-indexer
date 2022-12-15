use crate::cli::CheckCommand;
use reqwest::{blocking::Client, StatusCode};
use serde_json::{to_string_pretty, value::Value, Map};
use std::process::Command;
use tracing::error;

fn center_align(s: &str, n: usize) -> String {
    format!("{: ^width$}", s, width = n)
}

fn rightpad_whitespace(s: &str, n: usize) -> String {
    format!("{:0width$}", s, width = n)
}

fn format_exec_msg(exec_name: &str, path: Option<String>) -> String {
    if path.is_some() {
        rightpad_whitespace(
            &format!("  Found '{}' at '{}'", exec_name, path.unwrap()),
            76,
        )
    } else {
        rightpad_whitespace(&format!("Could not located '{}'", exec_name), 76)
    }
}

fn find_executable_with_msg(exec_name: &str) -> (String, Option<String>, String) {
    let (emoji, path) = find_executable(exec_name);
    let p = path.clone();
    (emoji, path, format_exec_msg(exec_name, p))
}

fn find_executable(exec_name: &str) -> (String, Option<String>) {
    match Command::new("which").arg(exec_name).output() {
        Ok(o) => {
            let path = String::from_utf8_lossy(&o.stdout)
                .strip_suffix('\n')
                .map(|x| x.to_string());
            (center_align("✅", 3), path)
        }
        Err(e) => {
            error!("  Could not locate {}: {}", exec_name, e);
            (center_align("⛔️", 3), None)
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

fn find_executable_with_fallback_msg(
    exec_name: &str,
    fallback_exec_name: &str,
) -> (String, Option<String>, String) {
    let (emoji, path) = find_executable_with_fallback(exec_name, fallback_exec_name);
    let p = path.clone();
    (emoji, path, format_exec_msg(exec_name, p))
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
                center_align("✅", 3),
                center_align(&format!(
                    "Local fuel-indexer service found: PID({}) | Port({})",
                    &pid, &cmd.grpahql_api_port
                ), 76)
            ),
            None => (
                center_align("⛔️", 6),
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
                center_align("⛔️", 3),
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
    let fuel_core = "fuel-core";
    let docker = "docker";
    let fuelup = "fuelup";

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

            let res_json = res
                .json::<Map<String, Value>>()
                .expect("Failed to read JSON response.");

            println!(
                "\n✅ Sucessfully retrieved indexer service health:\n\n{}",
                to_string_pretty(&res_json).unwrap()
            );
        }
        Err(e) => {
            error!("\n❌ Could not connect to indexer service: {}", e);
        }
    }

    let (indexer_emoji, _indexer_path, indexer_msg) =
        find_executable_with_msg(fuel_indexer);
    let (psql_emoji, _psql_path, psql_msg) = find_executable_with_msg(psql);
    let (sqlite_emoji, _sqlite_path, sqlite_msg) =
        find_executable_with_fallback_msg(sqlite, sqlite3);
    let (fuel_core_emoji, _fuelcore_path, fuel_core_msg) =
        find_executable_with_msg(fuel_core);
    let (service_emoji, service_msg) = find_indexer_service_info(&command);
    let (docker_emoji, _docker_path, docker_msg) = find_executable_with_msg(docker);
    let (fuelup_emoji, _fuelup_path, fuelup_msg) = find_executable_with_msg(fuelup);

    let details_header = center_align("Details", 76);
    let check_header = center_align("Component", 20);
    let status_headers = center_align("Status", 5);
    let binary_header = rightpad_whitespace("fuel-indexer binary", 20);
    let service_header = rightpad_whitespace("fuel-indexer service", 20);
    let psql_header = rightpad_whitespace(psql, 20);
    let sqlite_header = rightpad_whitespace(sqlite, 20);
    let fuel_core_header = rightpad_whitespace(fuel_core, 20);
    let docker_header = rightpad_whitespace(docker, 20);
    let fuelup_header = rightpad_whitespace(fuelup, 20);

    let stdout = format!(
        r#"
+--------+------------------------+----------------------------------------------------------------------------+
| {status_headers} |  {check_header}  |{details_header}|
+--------+------------------------+----------------------------------------------------------------------------+
|  {indexer_emoji}  | {binary_header}   |{indexer_msg}|
+--------+------------------------+----------------------------------------------------------------------------+
| {service_emoji} | {service_header}   |{service_msg}|
+--------+------------------------+----------------------------------------------------------------------------+
|  {psql_emoji}  | {psql_header}   |{psql_msg}|
+--------+------------------------+----------------------------------------------------------------------------+
|  {sqlite_emoji}  | {sqlite_header}   |{sqlite_msg}|
+--------+------------------------+----------------------------------------------------------------------------+
|  {fuel_core_emoji}  | {fuel_core_header}   |{fuel_core_msg}|
+--------+------------------------+----------------------------------------------------------------------------+
|  {docker_emoji}  | {docker_header}   |{docker_msg}|
+--------+------------------------+----------------------------------------------------------------------------+
|  {fuelup_emoji}  | {fuelup_header}   |{fuelup_msg}|
+--------+------------------------+----------------------------------------------------------------------------+
"#
    );

    println!("{}", stdout);

    Ok(())
}
