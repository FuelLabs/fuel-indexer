#[allow(unused)]
pub(crate) use crate::commands::{
    auth::Command as AuthCommand, build::Command as BuildCommand,
    check::Command as CheckCommand, deploy::Command as DeployCommand,
    kill::Command as KillCommand, new::Command as NewCommand,
    remove::Command as RemoveCommand, start::Command as StartCommand,
    status::Command as StatusCommand,
};
use clap::{Parser, Subcommand};
use forc_tracing::{init_tracing_subscriber, TracingSubscriberOptions};

#[derive(Debug, Parser)]
#[clap(name = "forc index", about = "Fuel Indexer Orchestrator", version)]
pub struct Opt {
    /// The command to run
    #[clap(subcommand)]
    pub command: ForcIndex,
}

#[derive(Subcommand, Debug)]
pub enum ForcIndex {
    Auth(AuthCommand),
    Build(BuildCommand),
    Check(CheckCommand),
    Deploy(DeployCommand),
    Kill(KillCommand),
    New(NewCommand),
    Remove(RemoveCommand),
    Start(Box<StartCommand>),
    Status(StatusCommand),
}

pub async fn run_cli() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();
    let tracing_options = TracingSubscriberOptions {
        ..Default::default()
    };
    init_tracing_subscriber(tracing_options);

    match opt.command {
        ForcIndex::New(command) => crate::commands::new::exec(command),
        ForcIndex::Deploy(command) => crate::commands::deploy::exec(command).await,
        ForcIndex::Start(command) => crate::commands::start::exec(command).await,
        ForcIndex::Check(_command) => crate::commands::check::exec().await,
        ForcIndex::Remove(command) => crate::commands::remove::exec(command).await,
        ForcIndex::Build(command) => crate::commands::build::exec(command),
        ForcIndex::Auth(command) => crate::commands::auth::exec(command).await,
        ForcIndex::Kill(command) => crate::commands::kill::exec(command),
        ForcIndex::Status(command) => crate::commands::status::exec(command).await,
    }
}
