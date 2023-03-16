use env_logger::Builder;
use std::io::Write;

pub struct LoggerConfig {
    pub verbose: bool,
}

impl LoggerConfig {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn init(&self) {
        let mut builder = Builder::new();
        builder.format(|buf, record| {
            writeln!(buf, "{}: - {}", record.level(), record.args())
        });
        if self.verbose {
            builder.filter(None, log::LevelFilter::Debug);
        } else {
            builder.filter(None, log::LevelFilter::Info);
        }
        builder.init();
    }
}
