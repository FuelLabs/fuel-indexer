use crate::cli::StatusCommand;
use serde_json::{to_string_pretty, value::Value, Map};
use tracing::{error, info};

pub async fn status(StatusCommand { url }: StatusCommand) -> anyhow::Result<()> {
    let target = format!("{url}/api/status");

    match reqwest::get(&target).await {
        Ok(res) => {
            if res.status() != reqwest::StatusCode::OK {
                error!(
                    "\n❌ {target} returned a non-200 response code: {:?}",
                    res.status()
                );
                return Ok(());
            }

            let res_json = res
                .json::<Vec<Map<String, Value>>>()
                .await
                .expect("Failed to read JSON response.");

            info!("\n✅ Indexers:\n\n{}", to_string_pretty(&res_json).unwrap());
        }
        Err(e) => {
            error!("\n❌ Could not connect to indexer service:\n'{e}'");
        }
    }

    Ok(())
}
