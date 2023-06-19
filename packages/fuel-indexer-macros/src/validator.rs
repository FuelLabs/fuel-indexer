use crate::{constants::*, helpers::*};

pub struct GraphQLSchemaValidator;

impl GraphQLSchemaValidator {
    /// Check that the given name is not a reserved object name.
    pub fn check_disallowed_graphql_typedef_name(name: &str) {
        if DISALLOWED_OBJECT_NAMES.contains(name) {
            panic!("Object name '{name}' is reserved.",);
        }
    }

    pub fn check_disallowed_abi_typedef_name(name: &str) {
        if FUEL_PRIMITIVES.contains(name) {
            panic!("Object name '{name}' is reserved.",);
        }
    }
}
