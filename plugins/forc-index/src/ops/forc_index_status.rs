use crate::cli::StatusCommand;
use colorful::Color;
use colorful::Colorful;
use fuel_indexer_database_types::{IndexerStatus, IndexerStatusKind, RegisteredIndexer};
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

            let result = res
                .json::<Map<String, Value>>()
                .await
                .expect("Failed to read JSON response.");

            info!("\n✅ {}:\n", "Successfully fetched service health".bold());

            let client_status = result
                .get("client_status")
                .and_then(|x| x.as_str())
                .unwrap_or("missing");
            let database_status = result
                .get("database_status")
                .and_then(|x| x.as_str())
                .unwrap_or("missing");
            let uptime = result
                .get("uptime")
                .and_then(|x| x.as_str())
                .and_then(|x| x.to_string().parse::<u64>().ok())
                .map(|x| {
                    humantime::format_duration(std::time::Duration::from_secs(x))
                        .to_string()
                })
                .unwrap_or("missing".to_string());

            let client_status = if client_status == "OK" {
                client_status.color(Color::Green)
            } else {
                client_status.color(Color::Red)
            };
            let database_status = if database_status == "OK" {
                database_status.color(Color::Green)
            } else {
                database_status.color(Color::Red)
            };
            info!("client status: {client_status}");
            info!("database status: {database_status}");
            info!("uptime: {}\n", uptime.color(Color::Yellow));
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

            let result: Vec<(RegisteredIndexer, IndexerStatus)> =
                res.json().await.expect("Failed to read JSON response.");

            println!("{}\n", "Indexers:".bold());
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

fn print_indexers(indexers: Vec<(RegisteredIndexer, IndexerStatus)>) {
    let mut groupped: Vec<Vec<(RegisteredIndexer, IndexerStatus)>> = {
        let mut ixs: BTreeMap<String, Vec<(RegisteredIndexer, IndexerStatus)>> =
            BTreeMap::new();
        for (i, status) in indexers.into_iter() {
            ixs.entry(i.namespace.clone())
                .or_default()
                .push((i, status));
        }
        ixs.into_values().collect()
    };
    // Ensure consistent ordering, by the identifier within each namespace
    for group in groupped.iter_mut() {
        group.sort_by(|x, y| x.0.identifier.partial_cmp(&y.0.identifier).unwrap());
    }
    // Ensure consistent ordering of namespaces
    groupped.sort_by(|x, y| x[0].0.namespace.partial_cmp(&y[0].0.namespace).unwrap());
    for (namespace_i, group) in groupped.iter().enumerate() {
        let namespace = group[0].0.namespace.clone();
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
        println!("{} {}", ng1, namespace.color(Color::Blue).bold());
        for (i, indexer) in group.iter().enumerate() {
            // indexer glyphs
            let (ig1, ig2) = if i != group.len() - 1 {
                ("├─", "|")
            } else {
                ("└─", " ")
            };
            let message = indexer
                .1
                .status_message
                .lines()
                .map(|x| format!("{ng2}  {ig2}      {x}"))
                .collect::<Vec<String>>()
                .join("\n");
            let status = if indexer.1.status_kind == IndexerStatusKind::Error {
                indexer.1.status_kind.to_string().color(Color::Red)
            } else {
                indexer.1.status_kind.to_string().color(Color::Green)
            };
            println!(
                "{}  {} {}",
                ng2,
                ig1,
                indexer.0.identifier.clone().color(Color::Blue).bold()
            );
            println!("{}  {}  • id: {}", ng2, ig2, indexer.0.id);
            println!("{}  {}  • created at: {}", ng2, ig2, indexer.0.created_at);
            println!("{}  {}  • pubkey: {:?}", ng2, ig2, indexer.0.pubkey);
            println!("{}  {}  • status: {}", ng2, ig2, status);
            println!("{}  {}  • status message:", ng2, ig2);
            if !message.is_empty() {
                println!("{message}");
            }
            println!("{}  {}", ng2, ig2);
        }
    }
}
