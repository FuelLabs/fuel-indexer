use crate::graphql::constants::*;
use async_graphql_parser::types::{FieldDefinition, TypeDefinition, TypeKind};
use std::collections::HashSet;

/// General container used to store a set of GraphQL schema validation functions.
pub struct GraphQLSchemaValidator;

impl GraphQLSchemaValidator {
    /// Check that the given name is not a reserved object name.
    pub fn check_disallowed_graphql_typedef_name(name: &str) {
        if DISALLOWED_OBJECT_NAMES.contains(name) {
            panic!("TypeDefinition name '{name}' is reserved.");
        }
    }

    /// Check the given `TypeDefinition` name is not a disallowed Sway ABI name.
    pub fn check_disallowed_abi_typedef_name(name: &str) {
        if FUEL_PRIMITIVES.contains(name) {
            panic!("TypeDefinition name '{name}' is reserved.");
        }
    }

    /// Check that a `TypeKind::Union(UnionType)`'s members are either all virtual, or all regular/non-virtual
    pub fn check_derived_union_is_well_formed(
        typ: &TypeDefinition,
        virtual_type_names: &mut HashSet<String>,
    ) {
        match &typ.kind {
            TypeKind::Union(u) => {
                let union_name = typ.name.to_string();
                let member_count = u.members.len();
                let virtual_member_count = u
                    .members
                    .iter()
                    .map(|m| {
                        let member_name = m.node.to_string();
                        if virtual_type_names.contains(&member_name) {
                            1
                        } else {
                            0
                        }
                    })
                    .sum::<usize>();

                let mut has_virtual_member = false;

                u.members.iter().for_each(|m| {
                    let member_name = m.node.to_string();
                    if let Some(name) = virtual_type_names.get(&member_name) {
                        virtual_type_names.insert(name.to_owned());
                        has_virtual_member = true;
                    }
                });

                if has_virtual_member {
                    virtual_type_names.insert(union_name.clone());

                    // All members of a union must all be regular or virtual
                    if virtual_member_count != member_count {
                        panic!("TypeDefinition(Union({union_name})) does not have consistent virtual/non-virtual members.");
                    }
                }
            }
            _ => panic!("`TypeKind::Union(UnionType)` expected."),
        }
    }

    /// Ensure that a derived union type's fields are all of a consistent, single type.
    pub fn derived_field_type_is_consistent(
        union_name: &str,
        field_name: &str,
        fields: &HashSet<String>,
    ) {
        if fields.contains(field_name) {
            panic!("Derived type from Union({union_name}) contains Field({field_name}) which does not have a consistent type across all members.");
        }
    }

    /// Ensure a `FieldDefinition` is not a reference to a nested list.
    pub fn ensure_fielddef_is_not_nested_list(f: &FieldDefinition) {
        let name = f.name.to_string();
        if f.ty.node.to_string().matches('[').count() > 1 {
            panic!("FieldDefinition({name}) is a nested list, which is not supported.");
        }
    }
}
