use fuel_abi_types::abi::program::{ProgramABI, TypeDeclaration};
use fuel_indexer_lib::{constants, graphql};
use std::collections::HashMap;

// Given a `TypeDeclaration` for an ABI enum, generate a corresponding GraphQL
// `enum`.
//
// We can only decode enums with all variants of type (). For example:
//
// pub enum SimpleEnum {
//     One: (),
//     Two: (),
//     Three: (),
// }
//
// can be converted to GraphQL:
//
// enum SimpleEnumEntity {
//     One
//     Two
//     Three
// }
fn decode_enum(types: &[TypeDeclaration], ty: &TypeDeclaration) -> Option<String> {
    let name = ty.type_field.strip_prefix("enum ").unwrap();

    let mut fields: Vec<String> = vec![];
    if let Some(ref components) = ty.components {
        for c in components {
            let ty = &types.get(c.type_id)?;
            if is_unit_type(ty) {
                fields.push(c.name.to_string());
            } else {
                return None;
            }
        }
    }

    let fields = fields
        .into_iter()
        .map(|s| "    ".to_string() + &s)
        .collect::<Vec<String>>()
        .join("\n");

    let output = format!("enum {name} {{\n{fields}\n}}");

    Some(output)
}

// Given a `TypeDeclaration` for an ABI struct, generate a corresponding GraphQL `type`.
fn decode_struct(
    scalar_types: &HashMap<&str, &str>,
    abi_types: &[TypeDeclaration],
    ty: &TypeDeclaration,
) -> Option<String> {
    let name = ty.type_field.strip_prefix("struct ")?;

    let mut fields: Vec<String> = vec!["id: ID!".to_string()];
    if let Some(ref components) = ty.components {
        for c in components {
            // Skip the `id` field since we are inserting out own.
            if c.name.as_str() == "id" {
                continue;
            }

            let ty = &abi_types.get(c.type_id)?.type_field;

            // Enum field.
            if let Some(ty) = ty.strip_prefix("enum ") {
                if constants::RESERVED_TYPEDEF_NAMES.contains(ty) {
                    // For reserved type names, we take the type as is.
                    fields.push(format!("{}: {}!", c.name, ty));
                } else {
                    // For generated types, we add a suffix -Entity.
                    fields.push(format!("{}: {}Entity!", c.name, ty))
                }
            // Struct field.
            } else if let Some(ty) = ty.strip_prefix("struct ") {
                if constants::RESERVED_TYPEDEF_NAMES.contains(ty) {
                    // For reserved type names, we take the type as is.
                    fields.push(format!("{}: {}!", c.name, ty));
                } else {
                    // For generated types, we add a suffix -Entity.
                    fields.push(format!("{}: {}Entity!", c.name, ty))
                }
            // Scalar field.
            } else if let Some(ty) = scalar_types.get(&ty.as_str()) {
                fields.push(format!("{}: {}!", c.name, ty));
            }
        }
    }

    let fields = fields
        .into_iter()
        .map(|s| "    ".to_string() + &s)
        .collect::<Vec<String>>()
        .join("\n");

    let output = format!("type {name}Entity @entity {{\n{fields}\n}}");

    Some(output)
}

// Generate a GraphQL schema from JSON ABI.
pub fn generate_schema(json_abi: &std::path::Path) -> String {
    let source =
        fuels_code_gen::utils::Source::parse(json_abi.to_string_lossy()).unwrap();
    let source = source.get().unwrap();
    let abi: ProgramABI = serde_json::from_str(&source).unwrap();
    let abi_types: Vec<TypeDeclaration> = abi
        .types
        .into_iter()
        .map(crate::helpers::strip_callpath_from_type_field)
        .collect();

    let mut output: Vec<String> = vec![];

    for ty in abi_types.iter() {
        // Skip all generic types
        if constants::IGNORED_GENERIC_METADATA.contains(ty.type_field.as_str()) {
            continue;
        }

        // Only generate schema types for structs and enums
        if let Some(name) = ty.type_field.strip_prefix("struct ") {
            if constants::RESERVED_TYPEDEF_NAMES.contains(name)
                || constants::GENERIC_STRUCTS.contains(name)
            {
                continue;
            }
            if let Some(result) =
                decode_struct(&graphql::constants::ABI_TYPE_MAP, &abi_types, ty)
            {
                output.push(result);
            }
        } else if let Some(name) = ty.type_field.strip_prefix("enum ") {
            if constants::RESERVED_TYPEDEF_NAMES.contains(name)
                || constants::GENERIC_STRUCTS.contains(name)
            {
                continue;
            }
            if let Some(result) = decode_enum(&abi_types, ty) {
                output.push(result);
            }
        }
    }

    output.join("\n\n")
}
