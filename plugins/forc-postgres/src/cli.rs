pub use crate::commands::{
    create::Command as CreateDbCommand, drop::Command as DropDbCommand,
    start::Command as StartDbCommand, stop::Command as StopDbCommand,
};
use clap::{Parser, Subcommand};
use forc_tracing::{init_tracing_subscriber, TracingSubscriberOptions};

#[derive(Debug, Parser)]
#[clap(name = "forc postgres", about = "Fuel Postgres Orchestrator", version)]
pub struct Opt {
    /// The command to run
    #[clap(subcommand)]
    pub command: ForcPostgres,
}

#[derive(Subcommand, Debug)]
pub enum ForcPostgres {
    Create(CreateDbCommand),
    Drop(DropDbCommand),
    Start(StartDbCommand),
    Stop(StopDbCommand),
}

pub async fn run_cli() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();
    let tracing_options = TracingSubscriberOptions {
        ..Default::default()
    };

    init_tracing_subscriber(tracing_options);

    match opt.command {
        ForcPostgres::Create(command) => crate::commands::create::exec(command).await,
        ForcPostgres::Stop(command) => crate::commands::stop::exec(command).await,
        ForcPostgres::Drop(command) => crate::commands::drop::exec(command).await,
        ForcPostgres::Start(command) => crate::commands::start::exec(command).await,
    }
}
