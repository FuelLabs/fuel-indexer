use env_logger::Builder;
use std::io::Write;

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
        // log::max_level() returns LevelFilter::Off by default when a logger is not initialized.
        // This check ensures that we don't panic due to multiple logger initializations.
        if log::max_level() == log::LevelFilter::Off {
            let mut builder = Builder::new();
            builder.format(|buf, record| {
                writeln!(buf, "{}: - {}", record.level(), record.args())
            });
            if self.verbose {
                builder.filter(None, log::LevelFilter::Debug);
            } else {
                builder.filter(None, log::LevelFilter::Off);
            }
            builder.init();
        }
    }
}
