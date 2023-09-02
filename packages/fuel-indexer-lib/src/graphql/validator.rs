use crate::{constants::*, graphql::MAX_FOREIGN_KEY_LIST_FIELDS};
use async_graphql_parser::types::{
    FieldDefinition, ObjectType, TypeDefinition, TypeKind,
};
use std::collections::HashSet;

/// General container used to store a set of GraphQL schema validation functions.
pub struct GraphQLSchemaValidator;

impl GraphQLSchemaValidator {
    /// Check that the given name is not a reserved object name.
    pub fn check_disallowed_graphql_typedef_name(name: &str) {
        if RESERVED_TYPEDEF_NAMES.contains(name) {
            panic!("TypeDefinition name '{name}' is reserved.");
        }
    }

    /// Check that a `TypeKind::Union(UnionType)`'s members are either all virtual, or all regular/non-virtual
    pub fn check_derived_union_virtuality_is_well_formed(
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
        types: &HashSet<String>,
    ) {
        if types.len() > 1 {
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

    /// Ensure a `FieldDefinition` with name `id` is of type `ID`.
    pub fn id_field_is_type_id(f: &FieldDefinition, obj_name: &str) {
        let name = f.name.to_string();
        let field_type = f.ty.node.to_string();
        // FIXME: Find some way to use IdCol here?
        if name == "id" && field_type != "ID!" {
            panic!("FieldDefinition({name}) on TypeDefinition({obj_name}) must be of type `ID!`. Found type `{field_type}`.");
        }
    }

    /// Ensure `TypeDefinition`s that are marked as virtual, don't contain an `id: ID` `FieldDefinition`.
    ///
    /// `id: ID!` fields are reserved for non-virtual entities only.
    pub fn virtual_type_has_no_id_field(o: &ObjectType, obj_name: &str) {
        let has_id_field = o.fields.iter().any(|f| {
            f.node.name.to_string() == "id" && f.node.ty.node.to_string() == "ID!"
        });
        if has_id_field {
            panic!("Virtual TypeDefinition({obj_name}) cannot contain an `id: ID!` FieldDefinition.");
        }
    }

    /// Ensure that any `FieldDefinition` that itself is a foreign relationship, does not contain
    /// a `@unique` directive.
    pub fn foreign_key_field_contains_no_unique_directive(
        f: &FieldDefinition,
        obj_name: &str,
    ) {
        let name = f.name.to_string();
        let has_unique_directive = f
            .directives
            .iter()
            .any(|d| d.node.name.to_string() == "unique");
        if has_unique_directive {
            panic!("FieldDefinition({name}) on TypeDefinition({obj_name}) cannot contain a `@unique` directive.");
        }
    }

    /// Ensure that a given `TypeDefiniton` does not contain more than `MAX_FOREIGN_KEY_LIST_FIELDS` many-to-many relationships.
    pub fn verify_m2m_relationship_count(obj_name: &str, m2m_field_count: usize) {
        if m2m_field_count > MAX_FOREIGN_KEY_LIST_FIELDS {
            panic!(
                "TypeDefinition({obj_name}) exceeds the allowed number of many-to-many` relationships. The maximum allowed is {MAX_FOREIGN_KEY_LIST_FIELDS}.",
            );
        }
    }

    /// Ensure that a `FieldDefinition` that is a many-to-many relationship only references parent `FieldDefinitions` that
    /// are of type `id: ID!`.
    pub fn m2m_fk_field_ref_col_is_id(
        f: &FieldDefinition,
        obj_name: &str,
        ref_coltype: &str,
        ref_colname: &str,
    ) {
        let name = f.name.to_string();
        if ref_coltype != "UID" || ref_colname != "id" {
            panic!(
                "FieldDefinition({name}) on TypeDefinition({obj_name}) is a many-to-many relationship where the inner scalar is of type `{ref_colname}: {ref_coltype}!`. However, only inner scalars of type `id: ID!` are allowed.",
            );
        }
    }
}
