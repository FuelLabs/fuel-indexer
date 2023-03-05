use crate::cli::AuthCommand;
use crate::commands::auth;

pub fn init(command: AuthCommand) -> anyhow::Result<()> {
    let _ = auth::exec(command);
    Ok(())
}
