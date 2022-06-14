use anyhow::Result;
use fuel_indexer::EntityResult;
use fuel_indexer_schema::type_id;
use serde::{Deserialize, Serialize};

pub type Handle = fn(Vec<Vec<u8>>) -> Result<Option<EntityResult>>;

#[derive(Debug)]
pub struct CustomHandler {
    pub event: ReceiptEvent,
    pub namespace: String,
    pub entity: String,
    pub handle: Handle,
    pub type_id: u64,
}

impl CustomHandler {
    pub fn new(event: ReceiptEvent, namespace: String, entity: String, handle: Handle) -> Self {
        Self {
            event,
            type_id: type_id(&namespace, &entity),
            namespace,
            entity,
            handle,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum ReceiptEvent {
    LogData,
    Log,
    ReturnData,
    Other,
}

impl From<String> for ReceiptEvent {
    fn from(e: String) -> Self {
        match &e[..] {
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
            ReceiptEvent::LogData => "LogData".to_owned(),
            ReceiptEvent::Log => "Log".to_owned(),
            ReceiptEvent::ReturnData => "ReturnDataa".to_owned(),
            _ => "Other".to_owned(),
        }
    }
}
