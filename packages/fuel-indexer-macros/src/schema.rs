use crate::{decoder::*, helpers::*};
use async_graphql_parser::types::{TypeDefinition, TypeKind, TypeSystemDefinition};
use fuel_indexer_lib::graphql::GraphQLSchema;
use fuel_indexer_lib::{
    graphql::ParsedGraphQLSchema, utils::local_repository_root, ExecutionSource,
};
use quote::quote;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Process a schema's type definition into the corresponding tokens for use in an indexer module.
fn process_type_def(
    parsed: &ParsedGraphQLSchema,
    typ: &TypeDefinition,
) -> Option<proc_macro2::TokenStream> {
    let tokens = match &typ.kind {
        TypeKind::Object(_o) => ObjectDecoder::from_typedef(typ, parsed).into(),
        TypeKind::Enum(_e) => EnumDecoder::from_typedef(typ, parsed).into(),
        TypeKind::Union(_u) => ObjectDecoder::from_typedef(typ, parsed).into(),
        _ => proc_macro_error::abort_call_site!(
            "Unrecognized TypeKind in GraphQL schema: {:?}",
            typ.kind
        ),
    };

    Some(tokens)
}

/// Process a schema definition into the corresponding tokens for use in an indexer module.
fn process_definition(
    schema: &ParsedGraphQLSchema,
    definition: &TypeSystemDefinition,
) -> Option<proc_macro2::TokenStream> {
    match definition {
        TypeSystemDefinition::Type(def) => process_type_def(schema, &def.node),
        TypeSystemDefinition::Schema(_def) => None,
        def => {
            panic!("Unhandled definition type: {def:?}");
        }
    }
}

/// Process user-supplied GraphQL schema into code for indexer module.
pub(crate) fn process_graphql_schema(
    namespace: String,
    identifier: String,
    schema_path: String,
    exec_source: ExecutionSource,
) -> proc_macro2::TokenStream {
    let namespace_tokens = const_item("NAMESPACE", &namespace);
    let identifer_tokens = const_item("IDENTIFIER", &identifier);

    let path = local_repository_root()
        .map(|p| Path::new(&p).join(schema_path.clone()))
        .unwrap_or_else(|| PathBuf::from(&schema_path));

    let mut file = File::open(&path)
        .map_err(|e| {
            proc_macro_error::abort_call_site!(
                "Could not open schema file {:?} {:?}",
                path,
                e
            )
        })
        .unwrap();

    let mut schema_content = String::new();
    file.read_to_string(&mut schema_content).expect("IO error");

    let schema = GraphQLSchema::new(schema_content);

    let version_tokens = const_item("VERSION", schema.version());

    let mut output = quote! {
        #namespace_tokens
        #identifer_tokens
        #version_tokens
    };

    let schema =
        ParsedGraphQLSchema::new(&namespace, &identifier, exec_source, Some(&schema))
            .expect("Failed to parse GraphQL schema.");

    for definition in schema.ast().clone().definitions.iter() {
        if let Some(def) = process_definition(&schema, definition) {
            output = quote! {
                #output
                #def
            };
        }
    }

    output
}
