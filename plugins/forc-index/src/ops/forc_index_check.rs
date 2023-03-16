use crate::cli::CheckCommand;
use fuel_indexer_lib::{
    config::defaults,
    utils::{center_align, find_executable_with_msg, rightpad_whitespace},
};
use reqwest::{blocking::Client, StatusCode};
use serde_json::{to_string_pretty, value::Value, Map};
use std::process::Command;
use tracing::error;

fn find_indexer_service_info(grpahql_api_port: &str) -> (String, String) {
    let (emoji, msg) = match Command::new("lsof")
        .arg(&format!("-ti:{grpahql_api_port}"))
        .output()
    {
        Ok(o) => {
            let (emoji, msg) = match String::from_utf8_lossy(&o.stdout)
                .to_string()
                .strip_suffix('\n')
            {
                Some(pid) => (
                    center_align("✅", defaults::SUCCESS_EMOJI_PADDING),
                    rightpad_whitespace(
                        &format!(
                            "Local service found: PID({pid}) | Port({grpahql_api_port})."
                        ),
                        defaults::MESSAGE_PADDING,
                    ),
                ),
                None => (
                    center_align("⛔️", defaults::FAIL_EMOJI_PADDING),
                    rightpad_whitespace(
                        &format!("Failed to detect service at Port({grpahql_api_port}).",),
                        defaults::MESSAGE_PADDING,
                    ),
                ),
            };

            (emoji, msg)
        }
        Err(_e) => (
            center_align("⛔️", defaults::FAIL_EMOJI_PADDING),
            rightpad_whitespace(
                &format!("Failed to detect service at Port({grpahql_api_port}).",),
                defaults::MESSAGE_PADDING,
            ),
        ),
    };

    (emoji, msg)
}

pub fn init(command: CheckCommand) -> anyhow::Result<()> {
    let CheckCommand {
        url,
        grpahql_api_port,
    } = command;

    let target = format!("{url}/api/health");
    let psql = "psql";
    let fuel_indexer = "fuel-indexer";
    let fuel_core = "fuel-core";
    let docker = "docker";
    let fuelup = "fuelup";
    let wasm_snip = "wasm-snip";
    let forc_pg = "forc-postgres";
    let rustc = "rustc";
    let forc_wallet = "forc-wallet";

    match Client::new().get(&target).send() {
        Ok(res) => {
            if res.status() != StatusCode::OK {
                error!(
                    "\n❌ {target} returned a non-200 response code: {:?}",
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
            error!("\n❌ Could not connect to indexer service:\n'{e}'");
        }
    }

    let (indexer_emoji, _indexer_path, indexer_msg) =
        find_executable_with_msg(fuel_indexer);
    let (psql_emoji, _psql_path, psql_msg) = find_executable_with_msg(psql);
    let (fuel_core_emoji, _fuelcore_path, fuel_core_msg) =
        find_executable_with_msg(fuel_core);
    let (service_emoji, service_msg) = find_indexer_service_info(&grpahql_api_port);
    let (docker_emoji, _docker_path, docker_msg) = find_executable_with_msg(docker);
    let (fuelup_emoji, _fuelup_path, fuelup_msg) = find_executable_with_msg(fuelup);
    let (forc_pg_emoji, _forc_pg_path, forc_pg_msg) = find_executable_with_msg(fuelup);
    let (wasm_snip_emoji, _wasm_snip_path, wasm_snip_msg) =
        find_executable_with_msg(wasm_snip);
    let (rustc_emoji, _rustc_path, rustc_msg) = find_executable_with_msg(rustc);
    let (forc_wallet_emoji, _forc_wallet_path, forc_wallet_msg) =
        find_executable_with_msg(forc_wallet);

    // Padding here is done on an as-needed basis
    let status_padding = 5;
    let details_header = center_align("Details", defaults::MESSAGE_PADDING + 2);
    let check_header = center_align("Component", defaults::HEADER_PADDING);
    let status_headers = center_align("Status", status_padding);
    let binary_header =
        rightpad_whitespace("fuel-indexer binary", defaults::HEADER_PADDING);
    let service_header =
        rightpad_whitespace("fuel-indexer service", defaults::HEADER_PADDING);
    let psql_header = rightpad_whitespace(psql, defaults::HEADER_PADDING);
    let fuel_core_header = rightpad_whitespace(fuel_core, defaults::HEADER_PADDING);
    let docker_header = rightpad_whitespace(docker, defaults::HEADER_PADDING);
    let fuelup_header = rightpad_whitespace(fuelup, defaults::HEADER_PADDING);
    let wasm_snip_header = rightpad_whitespace(wasm_snip, defaults::HEADER_PADDING);
    let forc_pg_header = rightpad_whitespace(forc_pg, defaults::HEADER_PADDING);
    let rustc_header = rightpad_whitespace(rustc, defaults::HEADER_PADDING);
    let forc_wallet_header = rightpad_whitespace(forc_wallet, defaults::HEADER_PADDING);

    let stdout = format!(
        r#"
+--------+------------------------+---------------------------------------------------------+
| {status_headers} |  {check_header}  |{details_header}|
+--------+------------------------+---------------------------------------------------------+
|  {indexer_emoji}  | {binary_header}   |  {indexer_msg}|
+--------+------------------------+---------------------------------------------------------+
| {service_emoji} | {service_header}   |  {service_msg}|
+--------+------------------------+---------------------------------------------------------+
|  {psql_emoji}  | {psql_header}   |  {psql_msg}|
+--------+------------------------+---------------------------------------------------------+
|  {fuel_core_emoji}  | {fuel_core_header}   |  {fuel_core_msg}|
+--------+------------------------+---------------------------------------------------------+
|  {docker_emoji}  | {docker_header}   |  {docker_msg}|
+--------+------------------------+---------------------------------------------------------+
|  {fuelup_emoji}  | {fuelup_header}   |  {fuelup_msg}|
+--------+------------------------+---------------------------------------------------------+
|  {wasm_snip_emoji}  | {wasm_snip_header}   |  {wasm_snip_msg}|
+--------+------------------------+---------------------------------------------------------+
|  {forc_pg_emoji}  | {forc_pg_header}   |  {forc_pg_msg}|
+--------+------------------------+---------------------------------------------------------+
|  {rustc_emoji}  | {rustc_header}   |  {rustc_msg}|
+--------+------------------------+---------------------------------------------------------+
|  {forc_wallet_emoji}  | {forc_wallet_header}   |  {forc_wallet_msg}|
+--------+------------------------+---------------------------------------------------------+
"#
    );

    println!("{stdout}");

    Ok(())
}
