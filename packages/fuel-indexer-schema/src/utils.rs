extern crate alloc;

use crate::{parser::ParsedGraphQLSchema, IndexerSchemaError, IndexerSchemaResult};
use alloc::vec::Vec;
use async_graphql_parser::types::{
    BaseType, Directive, FieldDefinition, ServiceDocument, Type, TypeDefinition,
    TypeKind, TypeSystemDefinition,
};
use fuel_indexer_database_types::{directives, ColumnType, IdCol};
use fuel_indexer_types::graphql::{GraphqlObject, IndexMetadata};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

pub const BASE_SCHEMA: &str = include_str!("./base.graphql");
pub const JOIN_DIRECTIVE_NAME: &str = "join";
pub const UNIQUE_DIRECTIVE_NAME: &str = "unique";
pub const INDEX_DIRECTIVE_NAME: &str = "indexed";
pub const NORELATION_DIRECTIVE_NAME: &str = "norelation";

type ForeignKeyMap = HashMap<String, HashMap<String, (String, String)>>;

/// Inject the default GraphQL entities used by the indexer into the user's GraphQL schema.
pub fn inject_native_entities_into_schema(schema: &str) -> String {
    format!("{}{}", schema, IndexMetadata::schema_fragment())
}

/// Remove special chars from GraphQL field type name.
pub fn normalize_field_type_name(name: &str) -> String {
    name.replace(['!', '[', ']'], "")
}

/// Convert GraphQL field type name to SQL table name.
pub fn field_type_table_name(f: &FieldDefinition) -> String {
    normalize_field_type_name(&f.ty.to_string()).to_lowercase()
}

/// Returns a SHA-256 hash of a schema's content.
pub fn schema_version(schema: &str) -> String {
    format!("{:x}", Sha256::digest(schema.as_bytes()))
}

/// Returns the name of a type in a type defintion.
pub fn type_name(typ: &TypeDefinition) -> String {
    typ.name.clone().to_string()
}

/// Returns the index directive of a field, if present.
pub fn get_index_directive(field: &FieldDefinition) -> Option<directives::Index> {
    let FieldDefinition {
        directives,
        name: field_name,
        ..
    } = field.clone();

    let mut directives: Vec<Directive> = directives
        .into_iter()
        .map(|d| d.into_inner().into_directive())
        .collect();

    if directives.len() == 1 {
        let Directive { name, .. } = directives.pop().unwrap();
        if name.to_string().as_str() == INDEX_DIRECTIVE_NAME {
            return Some(directives::Index::new(field_name.to_string()));
        }
    }

    None
}

pub fn get_unique_directive(field: &FieldDefinition) -> directives::Unique {
    let FieldDefinition { directives, .. } = field.clone();
    let mut directives: Vec<Directive> = directives
        .into_iter()
        .map(|d| d.into_inner().into_directive())
        .collect();

    if directives.len() == 1 {
        let Directive { name, .. } = directives.pop().unwrap();
        if name.to_string().as_str() == UNIQUE_DIRECTIVE_NAME {
            return directives::Unique(true);
        }
    }

    directives::Unique(false)
}

/// Given a field whos type references another object in the schema (i.e.,
/// a foreign key field), return metadata about the field and the referenced object.
///
/// This should only be called on `ColumnType::ForeignKey`.
pub fn get_join_directive_info(
    field: &FieldDefinition,
    type_name: &String,
    types_map: &HashMap<String, String>,
) -> directives::Join {
    let FieldDefinition {
        name: field_name,
        directives,
        ..
    } = field.clone();

    let mut directives: Vec<Directive> = directives
        .into_iter()
        .map(|d| d.into_inner().into_directive())
        .collect();

    let field_type_name = normalize_field_type_name(&field.ty.to_string());

    let (reference_field_name, ref_field_type_name) = if directives.len() == 1 {
        let Directive {
            mut arguments,
            name: directive_name,
            ..
        } = directives.pop().unwrap();

        assert_eq!(
            directive_name.to_string().as_str(),
            JOIN_DIRECTIVE_NAME,
            "Cannot call get_join_directive_info on a non-foreign key item."
        );

        let (_, ref_field_name) = arguments.pop().unwrap();

        let field_id = format!("{field_type_name}.{ref_field_name}");

        let ref_field_type_name = types_map
            .get(&field_id)
            .unwrap_or_else(|| {
                panic!("Explicit foreign key field '{field_id}' is not defined in the schema.",)
            })
            .to_owned();

        (ref_field_name.to_string(), ref_field_type_name)
    } else {
        let ref_field_name = IdCol::to_lowercase_string();
        let field_id = format!("{type_name}.{ref_field_name}");
        let mut ref_field_type_name = types_map
            .get(&field_id)
            .unwrap_or_else(|| {
                panic!("Implicit foreign key field '{field_id}' is not defined in the schema.",)
            })
            .to_owned();

        // In the case where we have an Object! foreign key reference on a field,
        // if that object's default 'id' field is 'ID' then 'ID' is going to create
        // another primary key (can't do that in SQL) -- so we manually change that to
        // an integer type here. Might have to do this for foreign key directives (above)
        // as well
        let non_primary_key_int = ColumnType::UInt8.to_string();
        if ref_field_type_name == IdCol::to_uppercase_string() {
            ref_field_type_name = non_primary_key_int;
        }

        (ref_field_name, ref_field_type_name)
    };

    directives::Join {
        field_type_name,
        field_name: field_name.to_string(),
        reference_field_name,
        object_name: type_name.to_string(),
        reference_field_type_name: ref_field_type_name,
    }
}

/// Given a GraphQL document return a `HashMap` where each key in the map
/// is a the fully qualified field name, and each value in the map is the
/// Fuel type of the field (e.g., `UInt8`, `Address`, etc).
///
/// Each entry in the map represents a field
pub fn build_schema_fields_and_types_map(
    ast: &ServiceDocument,
) -> IndexerSchemaResult<HashMap<String, String>> {
    let mut types_map = HashMap::new();
    for def in ast.definitions.iter() {
        if let TypeSystemDefinition::Type(typ) = def {
            match &typ.node.kind {
                TypeKind::Scalar => {}
                TypeKind::Enum(e) => {
                    let name = &typ.node.name.to_string();
                    for val in &e.values {
                        let val_name = &val.node.value.to_string();
                        let val_id = format!("{name}.{val_name}");
                        types_map.insert(val_id, name.to_string());
                    }
                }
                TypeKind::Object(obj) => {
                    for field in &obj.fields {
                        let field = &field.node;
                        let field_type = field.ty.to_string().replace('!', "");
                        let obj_name = &typ.node.name.to_string();
                        let field_name = &field.name.to_string();
                        let field_id = format!("{obj_name}.{field_name}");
                        types_map.insert(field_id, field_type);
                    }
                }
                _ => {
                    return Err(IndexerSchemaError::UnsupportedTypeKind);
                }
            }
        }
    }

    Ok(types_map)
}

/// Given a GraphQL document, return a two `HashSet`s - one for each
/// unique field type, and one for each unique directive.
pub fn build_schema_types_set(
    ast: &ServiceDocument,
) -> (HashSet<String>, HashSet<String>) {
    let types: HashSet<String> = ast
        .definitions
        .iter()
        .filter_map(|def| {
            if let TypeSystemDefinition::Type(typ) = def {
                Some(&typ.node)
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
            if let TypeSystemDefinition::Directive(dir) = def {
                Some(dir.node.name.to_string())
            } else {
                None
            }
        })
        .collect();

    (types, directives)
}

/// Iterates through a schema's type defintions and returns a map of entities with
/// their foreign keys.
pub fn get_foreign_keys(
    namespace: &str,
    identifier: &str,
    is_native: bool,
    schema: &str,
) -> IndexerSchemaResult<ForeignKeyMap> {
    let parsed_schema =
        ParsedGraphQLSchema::new(namespace, identifier, is_native, Some(schema))?;

    let mut fks: ForeignKeyMap = HashMap::new();

    for def in parsed_schema.ast.definitions.iter() {
        if let TypeSystemDefinition::Type(t) = def {
            if let TypeKind::Object(o) = &t.node.kind {
                // TODO: Add more ignorable types as needed - and use lazy_static!
                if t.node.name.to_string().to_lowercase() == *"queryroot" {
                    continue;
                }
                for field in o.fields.iter() {
                    let col_type = get_column_type(
                        &field.node.ty.node,
                        &parsed_schema.scalar_names,
                    )?;
                    #[allow(clippy::single_match)]
                    match col_type {
                        ColumnType::ForeignKey => {
                            let directives::Join {
                                reference_field_name,
                                ..
                            } = get_join_directive_info(
                                &field.node,
                                &t.node.name.to_string(),
                                &parsed_schema.field_type_mappings,
                            );

                            let fk = fks.get_mut(&t.node.name.to_string().to_lowercase());
                            match fk {
                                Some(fks_for_field) => {
                                    fks_for_field.insert(
                                        field.node.name.to_string(),
                                        (
                                            field_type_table_name(&field.node),
                                            reference_field_name.clone(),
                                        ),
                                    );
                                }
                                None => {
                                    let fks_for_field = HashMap::from([(
                                        field.node.name.to_string(),
                                        (
                                            field_type_table_name(&field.node),
                                            reference_field_name.clone(),
                                        ),
                                    )]);
                                    fks.insert(
                                        t.node.name.to_string().to_lowercase(),
                                        fks_for_field,
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(fks)
}

/// Given a GraphQL field, return its associated `ColumnType`.
pub fn get_column_type(
    field_type: &Type,
    primitives: &HashSet<String>,
) -> IndexerSchemaResult<ColumnType> {
    match &field_type.base {
        BaseType::Named(t) => {
            if !primitives.contains(t.as_str()) {
                return Ok(ColumnType::ForeignKey);
            }
            Ok(ColumnType::from(t.as_str()))
        }
        BaseType::List(_) => Err(IndexerSchemaError::ListTypesUnsupported),
    }
}

/// Get directive determining whether or not field's object should not be used to create SQL tables.
pub fn get_notable_directive_info(
    field: &FieldDefinition,
) -> IndexerSchemaResult<directives::NoRelation> {
    let FieldDefinition { directives, .. } = field.clone();

    let mut directives: Vec<Directive> = directives
        .into_iter()
        .map(|d| d.into_inner().into_directive())
        .collect();

    if directives.len() == 1 {
        let Directive { name, .. } = directives.pop().unwrap();
        if name.to_string().as_str() == NORELATION_DIRECTIVE_NAME {
            return Ok(directives::NoRelation(true));
        }
    }

    Ok(directives::NoRelation(false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql_parser::parse_schema;

    #[test]
    fn test_build_schema_fields_and_types_map_properly_builds_schema_types_map() {
        let schema = r#"
type BlockData {
    id: Bytes32! @unique
    height: UInt8!
    timestamp: Int8!
    gas_limit: UInt8!
    extra_data: MessageId!
}

type Tx {
    id: Bytes32! @unique
    block: BlockData!
    timestamp: Int8!
    status: Json!
    value: UInt8!
    tokens_transferred: Json!
}

type Account {
    address: Address!
}

type Contract {
    creator: ContractId!
}
        "#;

        let ast = parse_schema(schema).unwrap();
        let types_map = build_schema_fields_and_types_map(&ast).unwrap();

        assert_eq!(
            *types_map.get("BlockData.id").unwrap(),
            "Bytes32".to_string()
        );
        assert_eq!(*types_map.get("Tx.block").unwrap(), "BlockData".to_string());
        assert_eq!(
            *types_map.get("Account.address").unwrap(),
            "Address".to_string()
        );
        assert_eq!(
            *types_map.get("Contract.creator").unwrap(),
            "ContractId".to_string()
        );
        assert_eq!(
            *types_map.get("BlockData.extra_data").unwrap(),
            "MessageId".to_string()
        );
        assert_eq!(types_map.get("BlockData.doesNotExist"), None);
    }

    #[test]
    fn test_build_schema_objects_set_returns_proper_schema_types_set() {
        let schema = r#"
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

        let ast = parse_schema(schema).unwrap();

        let (obj_set, _directives_set) = build_schema_types_set(&ast);

        assert!(!obj_set.contains("NotARealThing"));
        assert!(obj_set.contains("Borrower"));
        assert!(obj_set.contains("Auditor"));
    }
}
