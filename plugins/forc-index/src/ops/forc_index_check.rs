use crate::cli::CheckCommand;
use reqwest::{blocking::Client, StatusCode};
use serde_json::{to_string_pretty, value::Value, Map};
use std::process::Command;
use tracing::error;

const MESSAGE_PADDING: usize = 64;
const SUCCESS_EMOJI_PADDING: usize = 3;
const FAIL_EMOJI_PADDING: usize = 6;
const HEADER_PADDING: usize = 20;

fn center_align(s: &str, n: usize) -> String {
    format!("{s: ^n$}")
}

fn rightpad_whitespace(s: &str, n: usize) -> String {
    format!("{s:0n$}")
}

fn format_exec_msg(exec_name: &str, path: Option<String>) -> String {
    if let Some(path) = path {
        rightpad_whitespace(&path, MESSAGE_PADDING)
    } else {
        rightpad_whitespace(&format!("Can't locate {exec_name}"), MESSAGE_PADDING)
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
                .map(|x| x.to_string())
                .unwrap_or_else(String::new);

            if !path.is_empty() {
                (center_align("✅", SUCCESS_EMOJI_PADDING), Some(path))
            } else {
                (center_align("⛔️", FAIL_EMOJI_PADDING - 2), None)
            }
        }
        Err(_e) => (center_align("⛔️", FAIL_EMOJI_PADDING), None),
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
                .strip_suffix('\n')
            {
                Some(pid) => (
                    center_align("✅", SUCCESS_EMOJI_PADDING),
                    rightpad_whitespace(
                        &format!(
                            "Local service found: PID({}) | Port({}).",
                            &pid, &cmd.grpahql_api_port
                        ),
                        MESSAGE_PADDING,
                    ),
                ),
                None => (
                    center_align("⛔️", FAIL_EMOJI_PADDING),
                    rightpad_whitespace(
                        &format!(
                            "Failed to detect service at Port({}).",
                            cmd.grpahql_api_port
                        ),
                        MESSAGE_PADDING,
                    ),
                ),
            };

            (emoji, msg)
        }
        Err(_e) => (
            center_align("⛔️", FAIL_EMOJI_PADDING),
            rightpad_whitespace(
                &format!(
                    "Failed to detect service at Port({}).",
                    cmd.grpahql_api_port
                ),
                MESSAGE_PADDING,
            ),
        ),
    };

    (emoji, msg)
}

pub fn init(command: CheckCommand) -> anyhow::Result<()> {
    let target = format!("{}/api/health", command.url);

    let psql = "psql";
    let fuel_indexer = "fuel-indexer";
    let fuel_core = "fuel-core";
    let docker = "docker";
    let fuelup = "fuelup";
    let wasm_snip = "wasm-snip";

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
                "\n✅ Sucessfully fetched service health:\n\n{}",
                to_string_pretty(&res_json).unwrap()
            );
        }
        Err(e) => {
            error!("\n❌ Could not connect to indexer service:\n'{}'", e);
        }
    }

    let (indexer_emoji, _indexer_path, indexer_msg) =
        find_executable_with_msg(fuel_indexer);
    let (psql_emoji, _psql_path, psql_msg) = find_executable_with_msg(psql);
    let (fuel_core_emoji, _fuelcore_path, fuel_core_msg) =
        find_executable_with_msg(fuel_core);
    let (service_emoji, service_msg) = find_indexer_service_info(&command);
    let (docker_emoji, _docker_path, docker_msg) = find_executable_with_msg(docker);
    let (fuelup_emoji, _fuelup_path, fuelup_msg) = find_executable_with_msg(fuelup);
    let (wasm_snip_emoji, _wasm_snip_path, wasm_snip_msg) =
        find_executable_with_msg(wasm_snip);

    // Padding here is done on an as-needed basis
    let status_padding = 5;
    let details_header = center_align("Details", MESSAGE_PADDING + 2);
    let check_header = center_align("Component", HEADER_PADDING);
    let status_headers = center_align("Status", status_padding);
    let binary_header = rightpad_whitespace("fuel-indexer binary", HEADER_PADDING);
    let service_header = rightpad_whitespace("fuel-indexer service", HEADER_PADDING);
    let psql_header = rightpad_whitespace(psql, HEADER_PADDING);
    let fuel_core_header = rightpad_whitespace(fuel_core, HEADER_PADDING);
    let docker_header = rightpad_whitespace(docker, HEADER_PADDING);
    let fuelup_header = rightpad_whitespace(fuelup, HEADER_PADDING);
    let wasm_snip_header = rightpad_whitespace(wasm_snip, HEADER_PADDING);

    let stdout = format!(
        r#"
+--------+------------------------+------------------------------------------------------------------+
| {status_headers} |  {check_header}  |{details_header}|
+--------+------------------------+------------------------------------------------------------------+
|  {indexer_emoji}  | {binary_header}   |  {indexer_msg}|
+--------+------------------------+------------------------------------------------------------------+
| {service_emoji} | {service_header}   |  {service_msg}|
+--------+------------------------+------------------------------------------------------------------+
|  {psql_emoji}  | {psql_header}   |  {psql_msg}|
+--------+------------------------+------------------------------------------------------------------+
|  {fuel_core_emoji}  | {fuel_core_header}   |  {fuel_core_msg}|
+--------+------------------------+------------------------------------------------------------------+
|  {docker_emoji}  | {docker_header}   |  {docker_msg}|
+--------+------------------------+------------------------------------------------------------------+
|  {fuelup_emoji}  | {fuelup_header}   |  {fuelup_msg}|
+--------+------------------------+------------------------------------------------------------------+
|  {wasm_snip_emoji}  | {wasm_snip_header}   |  {wasm_snip_msg}|
+--------+------------------------+------------------------------------------------------------------+
"#
    );

    println!("{stdout}");

    Ok(())
}
