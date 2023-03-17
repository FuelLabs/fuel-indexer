use crate::ops::forc_index_start;
use anyhow::Result;
use fuel_indexer_lib::config::IndexerArgs;

pub type Command = IndexerArgs;

pub async fn exec(command: Box<Command>) -> Result<()> {
    let _ = forc_index_start::init(*command).await;
    Ok(())
}
