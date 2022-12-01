pub mod abi;
pub mod ffi;
pub mod graphql;
pub mod tx;

use core::array::TryFromSliceError;
pub use fuel_types::{
    Address, AssetId, Bytes32, Bytes4, Bytes8, ContractId, MessageId, Salt, Word,
};
pub use fuels_core::{types::Bits256, Parameterize, Token, Tokenizable};
use fuels_types::{param_types::ParamType, errors::Error as SDKError, enum_variants::EnumVariants};
use serde::{Deserialize, Serialize};

pub type Error = Box<dyn std::error::Error>;
pub type ID = u64;
pub type Int4 = i32;
pub type Int8 = i64;
pub type UInt4 = u32;
pub type UInt8 = u64;
pub type Timestamp = u64;
pub type Charfield = String;

const IDENTITY_LEN: usize = 33;


// TODO: This is all copied over from the SDK cause can't wait
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Identity {
    Address(Address),
    ContractId(ContractId),
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
                    "Could not construct Identity from enum_selector. Received: {:?}",
                    enum_selector
                ))),
            }
        } else {
            Err(SDKError::InstantiationError(format!(
                "Could not construct Identity from token. Received: {:?}",
                token
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

impl From<[u8; 33]> for Identity {
    fn from(bytes: [u8; IDENTITY_LEN]) -> Self {
        match bytes.first() {
            Some(0u8) => Self::Address(Address::try_from(&bytes[1..]).expect("Failed")),
            Some(1u8) => {
                Self::ContractId(ContractId::try_from(&bytes[1..]).expect("Failed"))
            }
            _ => panic!("bar"),
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

impl TryFrom<&[u8]> for Identity {
    type Error = TryFromSliceError;

    fn try_from(bytes: &[u8]) -> Result<Identity, TryFromSliceError> {
        <[u8; IDENTITY_LEN]>::try_from(bytes).map(|b| b.into())
    }
}

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Json(pub String);
