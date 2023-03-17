use tracing::{level_filters::LevelFilter, subscriber::set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

/// Logger behavior based on the `verbose` flag:
///
/// * If `verbose` is set to `true`:
///   * The filter is set to `LevelFilter::Debug`, and the logger will print logs at the `Error`, `Warn`, `Info`, and `Debug` levels.
///
/// * If `verbose` is set to `false`:
///   * The filter is set to `LevelFilter::Off`, and the logger will not print any logs.
pub struct LoggerConfig {
    pub verbose: bool,
}

impl LoggerConfig {
    pub fn new(verbose: bool) -> Self {
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
