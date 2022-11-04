extern crate alloc;
use crate::directives;
use alloc::vec::Vec;
pub use fuel_indexer_database_types as sql_types;
use graphql_parser::schema::{
    Definition, Directive, Document, Field, ObjectType, TypeDefinition,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use strum::{AsRefStr, EnumString};

pub const BASE_SCHEMA: &str = include_str!("./base.graphql");
pub const JOIN_DIRECTIVE_NAME: &str = "join";
pub const UNIQUE_DIRECTIVE_NAME: &str = "unique";
pub const INDEX_DIRECTIVE_NAME: &str = "indexed";

#[derive(Debug, EnumString, AsRefStr, Default)]
pub enum IndexMethod {
    #[default]
    #[strum(serialize = "btree")]
    Btree,
    #[strum(serialize = "hash")]
    Hash,
}

pub fn normalize_field_type_name(name: &str) -> String {
    name.replace('!', "")
}

pub fn field_type_table_name(f: &Field<String>) -> String {
    normalize_field_type_name(&f.field_type.to_string()).to_lowercase()
}

pub struct IdCol {}
impl IdCol {
    pub fn to_lowercase_string() -> String {
        "id".to_string()
    }

    pub fn to_uppercase_string() -> String {
        "ID".to_string()
    }
}

// serde_scale for now, can look at other options if necessary.
pub fn serialize(obj: &impl Serialize) -> Vec<u8> {
    bincode::serialize(obj).expect("Serialize failed")
}

pub fn deserialize<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, String> {
    match bincode::deserialize(bytes) {
        Ok(obj) => Ok(obj),
        Err(e) => Err(format!("Bincode serde error {:?}", e)),
    }
}

pub fn schema_version(schema: &str) -> String {
    format!("{:x}", Sha256::digest(schema.as_bytes()))
}

pub fn type_name(typ: &TypeDefinition<String>) -> String {
    match typ {
        TypeDefinition::Scalar(obj) => obj.name.clone(),
        TypeDefinition::Object(obj) => obj.name.clone(),
        TypeDefinition::Interface(obj) => obj.name.clone(),
        TypeDefinition::Union(obj) => obj.name.clone(),
        TypeDefinition::Enum(obj) => obj.name.clone(),
        TypeDefinition::InputObject(obj) => obj.name.clone(),
    }
}

pub fn get_index_directive(field: &Field<String>) -> Option<directives::Index> {
    let Field {
        mut directives,
        name: field_name,
        ..
    } = field.clone();

    if directives.len() == 1 {
        let Directive { name, .. } = directives.pop().unwrap();
        if name == INDEX_DIRECTIVE_NAME {
            return Some(directives::Index::new(field_name));
        }
    }

    None
}

pub fn get_unique_directive(field: &Field<String>) -> directives::Unique {
    let Field { mut directives, .. } = field.clone();

    if directives.len() == 1 {
        let Directive { name, .. } = directives.pop().unwrap();
        if name == UNIQUE_DIRECTIVE_NAME {
            return directives::Unique(true);
        }
    }

    directives::Unique(false)
}

pub fn get_join_directive_info<'a>(
    field: &Field<'a, String>,
    obj: &ObjectType<'a, String>,
    types_map: &HashMap<String, String>,
) -> directives::Join {
    let Field {
        name: field_name,
        mut directives,
        ..
    } = field.clone();

    let field_type_name = normalize_field_type_name(&field.field_type.to_string());

    let (reference_field_name, ref_field_type_name) = if directives.len() == 1 {
        let Directive {
            mut arguments,
            name: directive_name,
            ..
        } = directives.pop().unwrap();

        assert_eq!(
            directive_name, JOIN_DIRECTIVE_NAME,
            "Cannot call get_join_directive_info on a non-foreign key item."
        );
        let (_, ref_field_name) = arguments.pop().unwrap();

        let field_id = format!("{}.{}", field_type_name, ref_field_name);

        let ref_field_type_name = types_map
            .get(&field_id)
            .unwrap_or_else(|| {
                panic!(
                    "Foreign key field '{}' is not defined in the schema.",
                    field_id
                )
            })
            .to_owned();

        (ref_field_name.to_string(), ref_field_type_name)
    } else {
        let ref_field_name = IdCol::to_lowercase_string();
        let field_id = format!("{}.{}", field_type_name, ref_field_name);
        let mut ref_field_type_name = types_map
            .get(&field_id)
            .unwrap_or_else(|| {
                panic!(
                    "Foreign key field '{}' is not defined in the schema.",
                    field_id
                )
            })
            .to_owned();

        // In the case where we have an Object! foreign key reference on a  field,
        // if that object's default 'id' field is 'ID' then 'ID' is going to create
        // another primary key (can't do that in SQL) -- so we manually change that to
        // an integer type here. Might have to do this for foreign key directives (above)
        // as well
        let non_primary_key_int = sql_types::ColumnType::UInt8.to_string();
        if ref_field_type_name == IdCol::to_uppercase_string() {
            ref_field_type_name = non_primary_key_int;
        }

        (ref_field_name, ref_field_type_name)
    };

    directives::Join {
        field_type_name,
        field_name,
        reference_field_name,
        object_name: obj.name.clone(),
        reference_field_type_name: ref_field_type_name,
    }
}

pub fn build_schema_fields_and_types_map(
    ast: &Document<String>,
) -> HashMap<String, String> {
    let mut types_map = HashMap::new();

    for def in ast.definitions.iter() {
        if let Definition::TypeDefinition(typ) = def {
            match typ {
                TypeDefinition::Object(obj) => {
                    for field in &obj.fields {
                        let field_type = field.field_type.to_string().replace('!', "");
                        let field_id = format!("{}.{}", obj.name, field.name);
                        types_map.insert(field_id, field_type);
                    }
                }
                o => panic!("Got a non-object type: '{:?}'", o),
            }
        }
    }

    types_map
}

pub fn build_schema_objects_set(
    ast: &Document<String>,
) -> (HashSet<String>, HashSet<String>) {
    let types: HashSet<String> = ast
        .definitions
        .iter()
        .filter_map(|def| {
            if let Definition::TypeDefinition(typ) = def {
                Some(typ)
            } else {
                None
            }
        })
        .map(type_name)
        .collect();

    let directives = ast
        .definitions
        .iter()
        .filter_map(|def| {
            if let Definition::DirectiveDefinition(dir) = def {
                Some(dir.name.clone())
            } else {
                None
            }
        })
        .collect();

    (types, directives)
}

#[cfg(test)]
mod tests {
    use super::*;
    use graphql_parser::parse_schema;

    #[test]
    fn test_build_schema_fields_and_types_map_properly_builds_schema_types_map() {
        let schema = r#"
schema {
    query: QueryRoot
}

type QueryRoot {
    block: Block
    tx: Tx
    count: Count
}

# https://ethereum.org/en/developers/docs/data-and-analytics/block-explorers/

type Block {
    id: Bytes32! @unique
    height: UInt8!
    timestamp: Int8!
    gas_limit: UInt8!
}

type Tx {
    id: Bytes32! @unique
    block: Block!
    timestamp: Int8!
    status: Jsonb!
    value: UInt8!
    tokens_transferred: Jsonb!
}

type Account {
    address: Address!
}

type Contract {
    creator: ContractId!
}
        "#;

        let ast = match parse_schema::<String>(schema) {
            Ok(ast) => ast,
            Err(e) => {
                panic!("Error parsing graphql schema {:?}", e)
            }
        };

        let types_map = build_schema_fields_and_types_map(&ast);

        assert_eq!(*types_map.get("Block.id").unwrap(), "Bytes32".to_string());
        assert_eq!(*types_map.get("Tx.block").unwrap(), "Block".to_string());
        assert_eq!(
            *types_map.get("Account.address").unwrap(),
            "Address".to_string()
        );
        assert_eq!(
            *types_map.get("Contract.creator").unwrap(),
            "ContractId".to_string()
        );
        assert_eq!(types_map.get("Block.doesNotExist"), None);
    }

    #[test]
    fn test_build_schema_objects_set_returns_proper_schema_types_set() {
        let schema = r#"
schema {
    query: QueryRoot
}

type QueryRoot {
    borrower: Borrower
    lender: Lender
    auditor: Auditor
}

type Borrower {
    account: Address! @indexed
}

type Lender {
    id: ID!
    borrower: Borrower! @join(on:account)
}

type Auditor {
    id: ID!
    account: Address!
    hash: Bytes32! @indexed
    borrower: Borrower! @join(on:account)
}
"#;

        let ast = match parse_schema::<String>(schema) {
            Ok(ast) => ast,
            Err(e) => {
                panic!("Error parsing graphql schema {:?}", e)
            }
        };

        let (obj_set, _directives_set) = build_schema_objects_set(&ast);

        assert!(obj_set.contains("QueryRoot"));
        assert!(!obj_set.contains("NotARealThing"));
        assert!(obj_set.contains("Borrower"));
        assert!(obj_set.contains("Auditor"));
    }
}
