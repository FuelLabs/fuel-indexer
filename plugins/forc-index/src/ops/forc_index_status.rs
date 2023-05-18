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
            print_indexers(&result);
        }
        Err(e) => {
            error!("\n❌ Could not connect to indexer service:\n'{e}'");
        }
    }

    Ok(())
}

fn print_indexers(indexers: &[RegisteredIndex]) {
    let groupped: Vec<&[RegisteredIndex]> = indexers
        .group_by(|x, y| x.namespace == y.namespace)
        .collect();
    for group in &groupped {
        let namespace = group[0].namespace.clone();
        println!("{}", namespace);
        for (i, indexer) in group.iter().enumerate() {
            let is_last = i == group.len() - 1;
            if !is_last {
                print!("├─ ");
            } else {
                print!("└─ ");
            }
            println!("{}", indexer.identifier);
            if !is_last {
                println!("|  • id: {}", indexer.id);
                println!("|  • created_at: {}", indexer.created_at);
                println!("|  • pubkey: {:?}", indexer.pubkey);
            } else {
                println!("   • id: {}", indexer.id);
                println!("   • created_at: {}", indexer.created_at);
                println!("   • pubkey: {:?}", indexer.pubkey);
            }
        }
    }
}
