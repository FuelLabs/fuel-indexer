use crate::graphql::constants::*;
use async_graphql_parser::types::{TypeDefinition, TypeKind};
use std::collections::HashSet;

/// General container used to store a set of schema/type validation functions.
pub struct GraphQLSchemaValidator;

impl GraphQLSchemaValidator {
    /// Check that the given name is not a reserved object name.
    pub fn check_disallowed_graphql_typedef_name(name: &str) {
        if DISALLOWED_OBJECT_NAMES.contains(name) {
            panic!("Object name '{name}' is reserved.",);
        }
    }

    /// Check the given `TypeDefinition` name is not a disallowed Sway ABI name.
    pub fn check_disallowed_abi_typedef_name(name: &str) {
        if FUEL_PRIMITIVES.contains(name) {
            panic!("Object name '{name}' is reserved.",);
        }
    }

    /// Check that a `TypeKind::Union(UnionType)`'s members are either all
    /// virtual, or all regular/non-virtual
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

                    // All members of a union must all be regualar or virtual
                    if virtual_member_count != member_count {
                        panic!("Union({union_name})'s members are not all virtual");
                    }
                }
            }
            _ => panic!("`TypeKind::Union(UnionType)` expected."),
        }
    }
}
