use crate::cli::StatusCommand;
use fuel_indexer_database_types::RegisteredIndex;
use tracing::error;

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

            let result = res
                .json::<Vec<RegisteredIndex>>()
                .await
                .expect("Failed to read JSON response.");
            print_indexers(result);
        }
        Err(e) => {
            error!("\n❌ Could not connect to indexer service:\n'{e}'");
        }
    }

    Ok(())
}

fn print_indexers(indexers: Vec<RegisteredIndex>) {
    let groupped: Vec<Vec<RegisteredIndex>> = {
        use std::collections::BTreeMap;
        let mut ixs: BTreeMap<String, Vec<RegisteredIndex>> = BTreeMap::new();
        for i in indexers.into_iter() {
            ixs.entry(i.namespace.clone()).or_insert(Vec::new()).push(i);
        }
        ixs.into_iter().map(|(_, x)| x).collect()
    };
    for (namespace_i, group) in groupped.iter().enumerate() {
        let namespace = group[0].namespace.clone();
        let is_last_namespace = namespace_i == groupped.len() - 1;
        // namespace glyphs
        let (ng1, ng2) = if namespace_i == 0 {
            // first
            ("┌─", "|")
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
            let (ig1, ig2) = if !(i == group.len() - 1) {
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
