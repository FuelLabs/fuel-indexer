use crate::cli::ForcIndex;
use forc_postgres::cli::ForcPostgres;
use tracing::{level_filters::LevelFilter, subscriber::set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

/// Logger behavior based on the `verbose` flag:
///
/// * If `verbose` is set to `true`:
///   * The filter is set to `LevelFilter::Info`, and the logger will print logs at the `Error`, `Warn`, and `Info` levels.
///
/// * If `verbose` is set to `false`:
///   * The filter is set to `LevelFilter::Error`, and the logger will only print logs at the `Error` level.
pub struct LoggerConfig {
    pub verbose: bool,
}

impl LoggerConfig {
    pub fn new(command: &ForcIndex) -> Self {
        let verbose = match command {
            ForcIndex::Init(c) => c.verbose,
            ForcIndex::New(c) => c.verbose,
            ForcIndex::Deploy(c) => c.verbose,
            ForcIndex::Start(c) => c.log_level == "info",
            ForcIndex::Check(_) => false,
            ForcIndex::Remove(c) => c.verbose,
            ForcIndex::Revert(c) => c.verbose,
            ForcIndex::Build(c) => c.verbose,
            ForcIndex::Auth(_) => false,
            ForcIndex::Postgres(opt) => match &opt.command {
                ForcPostgres::Create(c) => c.verbose,
                ForcPostgres::Stop(c) => c.verbose,
                ForcPostgres::Drop(c) => c.verbose,
                ForcPostgres::Start(c) => c.verbose,
            },
        };
        Self { verbose }
    }
    pub fn init(&self) {
        let level = if self.verbose {
            LevelFilter::INFO
        } else {
            LevelFilter::ERROR
        };

        let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

        if LogTracer::init().is_ok() {
            set_global_default(subscriber)
                .expect("Unable to set global tracing subscriber");
        }
    }
}
