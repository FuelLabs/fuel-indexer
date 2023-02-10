use crate::ops::forc_index_start;
use anyhow::Result;
use fuel_indexer_lib::config::IndexerArgs;

pub type Command = IndexerArgs;

pub fn exec(command: Box<Command>) -> Result<()> {
    forc_index_start::init(*command)?;
    Ok(())
}
