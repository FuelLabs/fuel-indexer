use crate::{database::Database, web::Web};

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
