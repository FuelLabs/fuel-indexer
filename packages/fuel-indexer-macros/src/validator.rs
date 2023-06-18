use crate::{constants::*, helpers::*};

pub struct GraphQLSchemaValidator;

impl GraphQLSchemaValidator {
    pub fn check_disallowed_typekind_name(name: &str) {
        if DISALLOWED_OBJECT_NAMES.contains(name) {
            panic!("Object name '{name}' is reserved.",);
        }
    }
}
