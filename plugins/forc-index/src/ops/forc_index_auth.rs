use crate::cli::AuthCommand;
use crate::commands::auth;
use std::path::Path;

pub fn init(command: AuthCommand) -> anyhow::Result<()> {
    let _ = auth::exec(command);
    Ok(())
}
