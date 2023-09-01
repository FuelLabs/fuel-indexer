use super::self_prelude::*;
use convert_case::{Case, Casing, StateConverter};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Name {
    Pascal(String),
    Snake(String),
    Camel(String),
}

impl Name {
    pub fn new_pascal(name: impl Into<String>) -> Self {
        Self::Pascal(name.into())
    }
    pub fn new_snake(name: impl Into<String>) -> Self {
        Self::Snake(name.into())
    }
    pub fn new_camel(name: impl Into<String>) -> Self {
        Self::Camel(name.into())
    }

    pub fn to_converter(&self) -> StateConverter<String> {
        match self {
            Name::Pascal(name) => name.from_case(Case::Pascal),
            Name::Snake(name) => name.from_case(Case::Snake),
            Name::Camel(name) => name.from_case(Case::Camel),
        }
    }
    pub fn to_pascal_string(&self) -> String {
        self.to_converter().to_case(Case::Pascal)
    }
    pub fn to_snake_string(&self) -> String {
        self.to_converter().to_case(Case::Snake)
    }
    pub fn to_camel_string(&self) -> String {
        self.to_converter().to_case(Case::Camel)
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Name::Pascal(name) => write!(f, "{}", name),
            Name::Snake(name) => write!(f, "{}", name),
            Name::Camel(name) => write!(f, "{}", name),
        }
    }
}
