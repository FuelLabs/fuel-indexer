use lazy_static::lazy_static;
pub(crate) mod db;
pub(crate) mod web;

pub use db::Database;
pub use web::Web;

pub trait Metric {
    fn init() -> Self;
}

pub struct Metrics {
    pub web: Web,
    pub db: Database,
}

impl Metric for Metrics {
    fn init() -> Self {
        Self {
            web: Web::init(),
            db: Database::init(),
        }
    }
}

lazy_static! {
    pub static ref METRICS: Metrics = Metrics::init();
}
