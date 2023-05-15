use crate::helpers::{const_item, generate_row_extractor};
use async_graphql_parser::parse_schema;
use async_graphql_parser::types::{
    BaseType, FieldDefinition, Type, TypeDefinition, TypeKind, TypeSystemDefinition,
};
use fuel_indexer_database_types::directives;
use fuel_indexer_lib::utils::local_repository_root;
use fuel_indexer_schema::utils::{
    build_schema_fields_and_types_map, build_schema_objects_set, get_join_directive_info,
    inject_native_entities_into_schema, schema_version, BASE_SCHEMA,
};
use fuel_indexer_types::type_id;
use lazy_static::lazy_static;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

lazy_static! {
    static ref COPY_TYPES: HashSet<&'static str> = HashSet::from([
        "Json",
        "Charfield",
        "Identity",
        "Blob",
        "Option<Json>",
        "Option<Blob>",
        "Option<Charfield>",
        "Option<Identity>"
    ]);
}

/// Process a schema type into its type tokens and its scalar type.
fn process_type(
    schema_types: &HashSet<String>,
    typ: &Type,
) -> (proc_macro2::TokenStream, proc_macro2::Ident) {
    match &typ.base {
        BaseType::Named(t) => {
            let name = t.to_string();
            if !schema_types.contains(&name) {
                panic!("Type '{name}' is undefined.",);
            }

            let id = format_ident! {"{}", name};

            if typ.nullable {
                (quote! { Option<#id> }, id)
            } else {
                (quote! { #id }, id)
            }
        }
        BaseType::List(_t) => panic!("Got a list type, we don't handle this yet..."),
    }
}

/// Process an object's field and return a group of tokens.
fn process_field(
    schema_types: &HashSet<String>,
    field_name: &String,
    field_type: &Type,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::Ident,
    proc_macro2::Ident,
    proc_macro2::TokenStream,
) {
    let (type_tokens, column_scalar_type) = process_type(schema_types, field_type);
    let ident = format_ident! {"{}", field_name.to_string()};

    let extractor = generate_row_extractor(
        ident.clone(),
        column_scalar_type.clone(),
        field_type.nullable,
    );

    (type_tokens, ident, column_scalar_type, extractor)
}

// Process a foreign key field of an object and return a group of tokens.
fn process_fk_field(
    schema_types: &HashSet<String>,
    object_name: String,
    field: &FieldDefinition,
    schema_types_map: &HashMap<String, String>,
    is_nullable: bool,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::Ident,
    proc_macro2::Ident,
    proc_macro2::TokenStream,
) {
    let directives::Join {
        field_name,
        reference_field_type_name,
        ..
    } = get_join_directive_info(field, &object_name, schema_types_map);

    let field_type_name = if !is_nullable {
        [reference_field_type_name, "!".to_string()].join("")
    } else {
        reference_field_type_name
    };
    let field_type: Type =
        Type::new(&field_type_name).expect("Could not construct type for processing");

    process_field(schema_types, &field_name, &field_type)
}

/// Process a schema's type definition into the corresponding tokens for use in an indexer module.
#[allow(clippy::too_many_arguments)]
fn process_type_def(
    namespace: &str,
    identifier: &str,
    schema_types: &HashSet<String>,
    typ: &TypeDefinition,
    processed: &mut HashSet<String>,
    primitives: &HashSet<String>,
    schema_types_map: &HashMap<String, String>,
    is_native: bool,
) -> Option<proc_macro2::TokenStream> {
    if let TypeKind::Object(obj) = &typ.kind {
        let object_name = typ.name.to_string();

        let type_id = type_id(&format!("{namespace}_{identifier}"), object_name.as_str());
        let mut block = quote! {};
        let mut row_extractors = quote! {};
        let mut construction = quote! {};
        let mut flattened = quote! {};

        for field in &obj.fields {
            let field_name = &field.node.name.to_string();
            let field_type = &field.node.ty.node;
            let (mut type_tokens, mut field_name, mut column_scalar_type, mut ext) =
                process_field(schema_types, field_name, field_type);

            let mut column_type_name = column_scalar_type.to_string();

            if processed.contains(&column_scalar_type.to_string())
                && !primitives.contains(&column_scalar_type.to_string())
            {
                (type_tokens, field_name, column_scalar_type, ext) = process_fk_field(
                    schema_types,
                    object_name.clone(),
                    &field.node,
                    schema_types_map,
                    field_type.nullable,
                );
                column_type_name = column_scalar_type.to_string();
            }

            processed.insert(column_type_name.clone());

            let clone = if COPY_TYPES.contains(column_type_name.as_str()) {
                quote! {.clone()}
            } else {
                quote! {}
            };

            let decoder = if field_type.nullable {
                quote! { FtColumn::#column_scalar_type(self.#field_name #clone), }
            } else {
                quote! { FtColumn::#column_scalar_type(Some(self.#field_name #clone)), }
            };

            block = quote! {
                #block
                #field_name: #type_tokens,
            };

            row_extractors = quote! {
                #ext
                #row_extractors
            };

            construction = quote! {
                #construction
                #field_name,
            };

            flattened = quote! {
                #flattened
                #decoder
            };
        }
        let strct = format_ident! {"{}", object_name};

        processed.insert(strct.to_string());

        if is_native {
            Some(quote! {
                #[derive(Debug, PartialEq, Eq, Hash)]
                pub struct #strct {
                    #block
                }

                #[async_trait::async_trait]
                impl Entity for #strct {
                    const TYPE_ID: i64 = #type_id;

                    fn from_row(mut vec: Vec<FtColumn>) -> Self {
                        #row_extractors
                        Self {
                            #construction
                        }
                    }

                    fn to_row(&self) -> Vec<FtColumn> {
                        vec![
                            #flattened
                        ]
                    }

                    async fn load(id: u64) -> Option<Self> {
                        unsafe {
                            match &db {
                                Some(d) => {
                                    match d.lock().await.get_object(Self::TYPE_ID, id).await {
                                        Some(bytes) => {
                                            let columns: Vec<FtColumn> = bincode::deserialize(&bytes).expect("Serde error.");
                                            let obj = Self::from_row(columns);
                                            Some(obj)
                                        },
                                        None => None,
                                    }
                                }
                                None => None,
                            }
                        }
                    }

                    async fn save(&self) {
                        unsafe {
                            match &db {
                                Some(d) => {
                                    d.lock().await.put_object(
                                        Self::TYPE_ID,
                                        self.to_row(),
                                        serialize(&self.to_row())
                                    ).await;
                                }
                                None => {},
                            }
                        }
                    }
                }
            })
        } else {
            Some(quote! {
                #[derive(Debug, PartialEq, Eq, Hash)]
                pub struct #strct {
                    #block
                }

                impl Entity for #strct {
                    const TYPE_ID: i64 = #type_id;

                    fn from_row(mut vec: Vec<FtColumn>) -> Self {
                        #row_extractors
                        Self {
                            #construction
                        }
                    }

                    fn to_row(&self) -> Vec<FtColumn> {
                        vec![
                            #flattened
                        ]
                    }
                }
            })
        }
    } else {
        panic!("Unexpected type: '{:?}'", typ.kind)
    }
}

#[allow(clippy::too_many_arguments)]
fn process_definition(
    namespace: &str,
    identifier: &str,
    types: &HashSet<String>,
    definition: &TypeSystemDefinition,
    processed: &mut HashSet<String>,
    primitives: &HashSet<String>,
    types_map: &HashMap<String, String>,
    is_native: bool,
) -> Option<proc_macro2::TokenStream> {
    match definition {
        TypeSystemDefinition::Type(def) => process_type_def(
            namespace, identifier, types, &def.node, processed, primitives, types_map,
            is_native,
        ),
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
    is_native: bool,
) -> proc_macro2::TokenStream {
    let path = match local_repository_root() {
        Some(p) => Path::new(&p).join(schema_path),
        None => PathBuf::from(&schema_path),
    };

    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            proc_macro_error::abort_call_site!(
                "Could not open schema file {:?} {:?}",
                path,
                e
            )
        }
    };

    let mut text = String::new();
    file.read_to_string(&mut text).expect("IO error");

    let text = inject_native_entities_into_schema(&text);

    let base_ast = match parse_schema(BASE_SCHEMA) {
        Ok(ast) => ast,
        Err(e) => {
            proc_macro_error::abort_call_site!("Error parsing graphql schema {:?}", e)
        }
    };

    let (primitives, _) = build_schema_objects_set(&base_ast);

    let ast = match parse_schema(&text) {
        Ok(ast) => ast,
        Err(e) => {
            proc_macro_error::abort_call_site!("Error parsing graphql schema {:?}", e)
        }
    };

    let (mut schema_types, _) = build_schema_objects_set(&ast);
    schema_types.extend(primitives.clone());

    let namespace_tokens = const_item("NAMESPACE", &namespace);
    let identifer_tokens = const_item("IDENTIFIER", &identifier);
    let version_tokens = const_item("VERSION", &schema_version(&text));

    let mut output = quote! {
        #namespace_tokens
        #identifer_tokens
        #version_tokens
    };

    let mut processed: HashSet<String> = HashSet::new();
    let schema_types_map: HashMap<String, String> =
        build_schema_fields_and_types_map(&ast).unwrap_or_else(|e| {
            panic!("Failed to build GraphQL schema field and types: {e}")
        });

    for definition in ast.definitions.iter() {
        if let Some(def) = process_definition(
            &namespace,
            &identifier,
            &schema_types,
            definition,
            &mut processed,
            &primitives,
            &schema_types_map,
            is_native,
        ) {
            output = quote! {
                #output
                #def
            };
        }
    }
    output
}
