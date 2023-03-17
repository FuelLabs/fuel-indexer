pub(crate) use crate::commands::{
    auth::Command as AuthCommand, build::Command as BuildCommand,
    check::Command as CheckCommand, deploy::Command as DeployCommand,
    init::Command as InitCommand, new::Command as NewCommand,
    remove::Command as RemoveCommand, revert::Command as RevertCommand,
    start::Command as StartCommand,
};
use crate::utils::log::LoggerConfig;
use clap::{Parser, Subcommand};
use forc_postgres::{
    cli::{ForcPostgres, Opt as ForcPostgresOpt},
    commands as pg_commands,
};
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
    Start(Box<StartCommand>),
    Check(CheckCommand),
    Remove(RemoveCommand),
    Revert(RevertCommand),
    Build(BuildCommand),
    Auth(AuthCommand),
    Postgres(ForcPostgresOpt),
}

pub async fn run_cli() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();
    let tracing_options = TracingSubscriberOptions {
        ..Default::default()
    };

    init_logger(&opt.command);
    //init_tracing_subscriber(tracing_options);

    match opt.command {
        ForcIndex::Init(command) => crate::commands::init::exec(command),
        ForcIndex::New(command) => crate::commands::new::exec(command),
        ForcIndex::Deploy(command) => crate::commands::deploy::exec(command),
        ForcIndex::Start(command) => crate::commands::start::exec(command).await,
        ForcIndex::Check(command) => crate::commands::check::exec(command),
        ForcIndex::Remove(command) => crate::commands::remove::exec(command),
        ForcIndex::Revert(command) => crate::commands::revert::exec(command).await,
        ForcIndex::Build(command) => crate::commands::build::exec(command),
        ForcIndex::Auth(command) => crate::commands::auth::exec(command),
        ForcIndex::Postgres(opt) => match opt.command {
            ForcPostgres::Create(command) => pg_commands::create::exec(command).await,
            ForcPostgres::Stop(command) => pg_commands::stop::exec(command).await,
            ForcPostgres::Drop(command) => pg_commands::drop::exec(command).await,
            ForcPostgres::Start(command) => pg_commands::start::exec(command).await,
        },
    }
}

fn init_logger(command: &ForcIndex) {
    let verbose = match command {
        ForcIndex::Init(c) => c.verbose,
        ForcIndex::New(c) => c.verbose,
        ForcIndex::Deploy(c) => c.verbose,
        ForcIndex::Start(_) => return,
        ForcIndex::Check(_) => return,
        ForcIndex::Remove(c) => c.verbose,
        ForcIndex::Revert(c) => c.verbose,
        ForcIndex::Build(c) => c.verbose,
        ForcIndex::Auth(_) => return,
        ForcIndex::Postgres(opt) => match &opt.command {
            ForcPostgres::Create(_) => return,
            ForcPostgres::Stop(_) => return,
            ForcPostgres::Drop(_) => return,
            ForcPostgres::Start(_) => return,
        },
    };

    println!("verbose: {}", verbose);

    let logger = LoggerConfig::new(verbose);
    logger.init();
}
