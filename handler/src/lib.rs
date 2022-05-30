use thiserror::Error;
use tracing::debug;

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("GenericError")]
    GenericError,
}

pub trait Handler {
    fn call(&self, data: Vec<Vec<u8>>) -> Result<(), HandlerError>;
}

pub struct Logger {}

impl Handler for Logger {
    fn call(&self, data: Vec<Vec<u8>>) -> Result<(), HandlerError> {
        debug!("Hanlder was called with '{:?}'", data);
        Ok(())
    }
}
