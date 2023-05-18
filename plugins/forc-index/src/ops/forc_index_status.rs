use crate::cli::StatusCommand;
use reqwest::{blocking::Client, StatusCode};
use serde_json::{to_string_pretty, value::Value, Map};
use tracing::{error, info};

pub fn status(StatusCommand { url }: StatusCommand) -> anyhow::Result<()> {
    let target = format!("{url}/api/status");

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
                .json::<Vec<Map<String, Value>>>()
                .expect("Failed to read JSON response.");

            info!("\n✅ Indexers:\n\n{}", to_string_pretty(&res_json).unwrap());
        }
        Err(e) => {
            error!("\n❌ Could not connect to indexer service:\n'{e}'");
        }
    }

    Ok(())
}
