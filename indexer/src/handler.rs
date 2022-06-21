use crate::{database::Database, IndexerResult};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

type Handle = fn(data: Vec<u8>, pg: Arc<Mutex<Database>>) -> IndexerResult<()>;

pub struct CustomHandler {
    pub event: ReceiptEvent,
    pub namespace: String,
    pub handle: Handle,
}

impl CustomHandler {
    pub fn new(event: ReceiptEvent, namespace: String, handle: Handle) -> Self {
        Self {
            event,
            namespace,
            handle,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum ReceiptEvent {
    // NOTE: Keeping these until https://github.com/FuelLabs/fuel-indexer/pull/65#discussion_r903138005 is figured out
    #[allow(non_camel_case_types)]
    an_event_name,
    #[allow(non_camel_case_types)]
    another_event_name,
    LogData,
    Log,
    ReturnData,
    Other,
}

impl From<String> for ReceiptEvent {
    fn from(e: String) -> Self {
        match &e[..] {
            "another_event_name" => ReceiptEvent::another_event_name,
            "an_event_name" => ReceiptEvent::an_event_name,
            "LogData" => ReceiptEvent::LogData,
            "Log" => ReceiptEvent::Log,
            "ReturnData" => ReceiptEvent::ReturnData,
            _ => ReceiptEvent::Other,
        }
    }
}

impl From<ReceiptEvent> for String {
    fn from(e: ReceiptEvent) -> String {
        match e {
            ReceiptEvent::another_event_name => "another_event_name".to_owned(),
            ReceiptEvent::an_event_name => "an_event_name".to_owned(),
            ReceiptEvent::LogData => "LogData".to_owned(),
            ReceiptEvent::Log => "Log".to_owned(),
            ReceiptEvent::ReturnData => "ReturnDataa".to_owned(),
            _ => "Other".to_owned(),
        }
    }
}
