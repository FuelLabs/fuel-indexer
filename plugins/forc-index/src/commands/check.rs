use crate::ops::forc_index_check;
use anyhow::Result;

pub async fn exec() -> Result<()> {
    forc_index_check::init().await
}
