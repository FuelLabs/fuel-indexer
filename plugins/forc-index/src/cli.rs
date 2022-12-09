pub(crate) use crate::commands::{
    deploy::Command as DeployCommand, init::Command as InitCommand,
    new::Command as NewCommand, start::Command as StartCommand,
    stop::Command as StopCommand, check::Command as CheckCommand,
};
use clap::{Parser, Subcommand};
use forc_tracing::{init_tracing_subscriber, TracingSubscriberOptions};

#[derive(Debug, Parser)]
#[clap(name = "forc index", about = "Fuel Index Orchestrator", version)]
struct Opt {
    /// The command to run
    #[clap(subcommand)]
    command: ForcIndex,
}

#[derive(Subcommand, Debug)]
enum ForcIndex {
    Init(InitCommand),
    New(NewCommand),
    Deploy(DeployCommand),
    Start(StartCommand),
    Check(CheckCommand),
    Stop(StopCommand),
}

pub async fn run_cli() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();
    let tracing_options = TracingSubscriberOptions {
        ..Default::default()
    };

    init_tracing_subscriber(tracing_options);

    match opt.command {
        ForcIndex::Init(command) => crate::commands::init::exec(command),
        ForcIndex::New(command) => crate::commands::new::exec(command),
        ForcIndex::Deploy(command) => crate::commands::deploy::exec(command),
        ForcIndex::Start(command) => crate::commands::start::exec(command),
<<<<<<< HEAD
        ForcIndex::Check(command) => crate::commands::check::exec(command),
=======
        ForcIndex::Stop(command) => crate::commands::stop::exec(command),
>>>>>>> 1d0bfb5440a36e1fa7d2a62a136c3739567663ee
    }
}
