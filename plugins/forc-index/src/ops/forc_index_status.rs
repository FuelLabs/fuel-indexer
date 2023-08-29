use crate::cli::StatusCommand;
use fuel_indexer_database_types::RegisteredIndexer;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONNECTION};
use serde_json::{to_string_pretty, value::Value, Map};
use std::collections::BTreeMap;
use tracing::{error, info};

pub async fn status(
    StatusCommand { url, auth, verbose }: StatusCommand,
) -> anyhow::Result<()> {
    let health_target = format!("{url}/api/health");
    let status_target = format!("{url}/api/status");

    let mut headers = HeaderMap::new();
    headers.insert(CONNECTION, "keep-alive".parse()?);
    if let Some(auth) = auth {
        headers.insert(AUTHORIZATION, auth.parse()?);
    }

    let client = reqwest::Client::new();

    match client.get(&health_target).send().await {
        Ok(res) => {
            if res.status() != reqwest::StatusCode::OK {
                error!(
                    "\n❌ {health_target} returned a non-200 response code: {:?}",
                    res.status()
                );
                return Ok(());
            }

            let res_json = res
                .json::<Map<String, Value>>()
                .await
                .expect("Failed to read JSON response.");

            info!(
                "\n✅ Sucessfully fetched service health:\n\n{}",
                to_string_pretty(&res_json).unwrap()
            );
        }
        Err(e) => {
            error!("\n❌ Could not connect to indexer service:\n'{e}'");
        }
    }

    match client.get(&status_target).headers(headers).send().await {
        Ok(res) => {
            let status = res.status();

            if status != reqwest::StatusCode::OK {
                if verbose {
                    error!(
                        "\n❌ Status check failed. {status_target} returned a non-200 response code: {:?}",
                        status
                    );
                }

                let result = res
                    .json::<Map<String, Value>>()
                    .await
                    .expect("Failed to read JSON response.");

                info!("\n{}", to_string_pretty(&result)?);
                return Ok(());
            }

            let result = res
                .json::<Vec<RegisteredIndexer>>()
                .await
                .expect("Failed to read JSON response.");

            print_indexers(result);
        }
        Err(e) => {
            if verbose {
                error!("\n❌ Status check failed. Could not connect to indexer service:\n'{e}'");
            } else {
                error!("\n❌ Status check failed.");
            }
        }
    }

    Ok(())
}

fn print_indexers(indexers: Vec<RegisteredIndexer>) {
    let groupped: Vec<Vec<RegisteredIndexer>> = {
        let mut ixs: BTreeMap<String, Vec<RegisteredIndexer>> = BTreeMap::new();
        for i in indexers.into_iter() {
            ixs.entry(i.namespace.clone()).or_default().push(i);
        }
        ixs.into_values().collect()
    };
    for (namespace_i, group) in groupped.iter().enumerate() {
        let namespace = group[0].namespace.clone();
        let is_last_namespace = namespace_i == groupped.len() - 1;
        // namespace glyphs
        let (ng1, ng2) = if namespace_i == 0 {
            // first and only
            if is_last_namespace {
                ("─", " ")
            // first
            } else {
                ("┌─", "|")
            }
        } else if !is_last_namespace {
            // middle
            ("├─", "|")
        } else {
            // last
            ("└─", " ")
        };
        println!("{} {}", ng1, namespace);
        for (i, indexer) in group.iter().enumerate() {
            // indexer glyphs
            let (ig1, ig2) = if i != group.len() - 1 {
                ("├─", "|")
            } else {
                ("└─", " ")
            };
            println!("{}  {} {}", ng2, ig1, indexer.identifier);
            println!("{}  {}  • id: {}", ng2, ig2, indexer.id);
            println!("{}  {}  • created_at: {}", ng2, ig2, indexer.created_at);
            println!("{}  {}  • pubkey: {:?}", ng2, ig2, indexer.pubkey);
        }
        if !is_last_namespace {
            println!("{}", ng2);
        }
    }
}
