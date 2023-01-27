use crate::{
    tx::{Transaction, TransactionStatus, TxId},
    type_id, Address, AssetId, Bytes32, ContractId, MessageId,
};
use core::array::TryFromSliceError;
pub use fuel_tx::Receipt;
pub use fuels_core::{Parameterize, Token, Tokenizable};
use fuels_types::{
    enum_variants::EnumVariants, errors::Error as SDKError, param_types::ParamType,
};
use serde::{Deserialize, Serialize};

pub const FUEL_TYPES_NAMESPACE: &str = "fuel";

pub trait NativeFuelType {
    fn type_id() -> usize;
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TransactionData {
    pub transaction: Transaction,
    pub status: TransactionStatus,
    pub receipts: Vec<Receipt>,
    pub id: TxId,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockData {
    pub height: u64,
    pub id: Bytes32,
    pub producer: Option<Bytes32>,
    pub time: i64,
    pub transactions: Vec<TransactionData>,
}

impl NativeFuelType for BlockData {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "BlockData") as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Transfer {
    pub contract_id: ContractId,
    pub to: ContractId,
    pub amount: u64,
    pub asset_id: AssetId,
    pub pc: u64,
    pub is: u64,
}

impl NativeFuelType for Transfer {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "Transfer") as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Log {
    pub contract_id: ContractId,
    pub ra: u64,
    pub rb: u64,
}

impl NativeFuelType for Log {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "Log") as usize
    }
}

// NOTE: Keeping for now, but I don't believe we need this.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LogData {
    pub contract_id: ContractId,
    pub data: Vec<u8>,
    pub rb: u64,
    pub len: u64,
    pub ptr: u64,
}

impl NativeFuelType for LogData {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "LogData") as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ScriptResult {
    pub result: u64,
    pub gas_used: u64,
}

impl NativeFuelType for ScriptResult {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "ScriptResult") as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TransferOut {
    pub contract_id: ContractId,
    pub to: Address,
    pub amount: u64,
    pub asset_id: AssetId,
    pub pc: u64,
    pub is: u64,
}

impl NativeFuelType for TransferOut {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "TransferOut") as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MessageOut {
    pub message_id: MessageId,
    pub sender: Address,
    pub recipient: Address,
    pub amount: u64,
    pub nonce: Bytes32,
    pub len: u64,
    pub digest: Bytes32,
    pub data: Vec<u8>,
}

impl NativeFuelType for MessageOut {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "MessageOut") as usize
    }
}

pub struct Return {
    pub contract_id: ContractId,
    pub val: u64,
    pub pc: u64,
    pub is: u64,
}

impl NativeFuelType for Return {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "Return") as usize
    }
}

const IDENTITY_LEN: usize = 33;

// TODO: https://github.com/FuelLabs/fuel-indexer/issues/386
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Identity {
    Address(fuel_tx::Address),
    ContractId(fuel_tx::ContractId),
}

impl Tokenizable for Identity {
    fn from_token(token: Token) -> Result<Self, SDKError> {
        if let Token::Enum(enum_selector) = token {
            match *enum_selector {
                (0u8, token, _) => Ok(Identity::Address(Address::from_token(token)?)),
                (1u8, token, _) => {
                    Ok(Identity::ContractId(ContractId::from_token(token)?))
                }
                (_, _, _) => Err(SDKError::InstantiationError(format!(
                    "Could not construct Identity from enum_selector. Received: {enum_selector:?}",
                ))),
            }
        } else {
            Err(SDKError::InstantiationError(format!(
                "Could not construct Identity from token. Received: {token:?}",
            )))
        }
    }
    fn into_token(self) -> Token {
        let (dis, tok) = match self {
            Self::Address(value) => (0u8, value.into_token()),
            Self::ContractId(value) => (1u8, value.into_token()),
        };
        if let ParamType::Enum { variants, .. } = Self::param_type() {
            let selector = (dis, tok, variants);
            Token::Enum(Box::new(selector))
        } else {
            panic!("should never happen as Identity::param_type() returns valid Enum variants");
        }
    }
}

impl Parameterize for Identity {
    fn param_type() -> ParamType {
        let param_types = vec![Address::param_type(), ContractId::param_type()];
        let variants = EnumVariants::new(param_types)
            .expect("should never happen as we provided valid Identity param types");
        ParamType::Enum {
            variants,
            generics: vec![],
        }
    }
}

impl From<[u8; IDENTITY_LEN]> for Identity {
    fn from(bytes: [u8; IDENTITY_LEN]) -> Self {
        match bytes.first() {
            Some(0u8) => Self::Address(Address::try_from(&bytes[1..]).expect("Failed")),
            Some(1u8) => {
                Self::ContractId(ContractId::try_from(&bytes[1..]).expect("Failed"))
            }
            _ => panic!("Failed"),
        }
    }
}

impl From<Identity> for [u8; IDENTITY_LEN] {
    fn from(salt: Identity) -> [u8; IDENTITY_LEN] {
        match salt {
            Identity::Address(v) => {
                let mut buff: [u8; IDENTITY_LEN] = [0u8; IDENTITY_LEN];
                buff[1..].copy_from_slice(v.as_ref());
                buff
            }
            Identity::ContractId(v) => {
                let mut buff: [u8; IDENTITY_LEN] = [1u8; IDENTITY_LEN];
                buff[1..].copy_from_slice(v.as_ref());
                buff
            }
        }
    }
}

impl From<Identity> for Address {
    fn from(ident: Identity) -> Address {
        match ident {
            Identity::Address(v) => v,
            _ => panic!("Conversion expects Address."),
        }
    }
}

impl From<Identity> for ContractId {
    fn from(ident: Identity) -> ContractId {
        match ident {
            Identity::ContractId(v) => v,
            _ => panic!("Conversion expects ContractId."),
        }
    }
}

impl TryFrom<&[u8]> for Identity {
    type Error = TryFromSliceError;

    fn try_from(bytes: &[u8]) -> Result<Identity, TryFromSliceError> {
        <[u8; IDENTITY_LEN]>::try_from(bytes).map(|b| b.into())
    }
}

impl From<fuels_core::Identity> for Identity {
    fn from(ident: fuels_core::Identity) -> Self {
        match ident {
            fuels_core::Identity::ContractId(v) => Self::ContractId(v),
            fuels_core::Identity::Address(v) => Self::Address(v),
        }
    }
}
