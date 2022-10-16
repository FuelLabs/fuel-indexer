use fuel_indexer_schema::{get_schema_types, schema_version, type_id, BASE_SCHEMA};
use graphql_parser::parse_schema;
use graphql_parser::schema::{
    Definition, Document, Field, SchemaDefinition, Type, TypeDefinition,
};
use quote::{format_ident, quote};
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;

fn process_type<'a>(
    types: &HashSet<String>,
    typ: &Type<'a, String>,
    nullable: bool,
) -> proc_macro2::TokenStream {
    match typ {
        Type::NamedType(t) => {
            if !types.contains(t) {
                panic!("Type {} is undefined.", t);
            }

            let id = format_ident! {"{}", t };

            if nullable {
                quote! { Option<#id> }
            } else {
                quote! { #id }
            }
        }
        Type::ListType(_t) => panic!("Got a list type, we don't handle this yet..."),
        Type::NonNullType(t) => process_type(types, t, false),
    }
}

fn process_field<'a>(
    types: &HashSet<String>,
    field: &Field<'a, String>,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::Ident,
    proc_macro2::TokenStream,
) {
    let Field {
        name, field_type, ..
    } = field;
    let typ = process_type(types, field_type, true);
    let ident = format_ident! {"{}", name};

    let extractor = quote! {
        let item = vec.pop().expect("Missing item in row");
        let #ident = match item {
            FtColumn::#typ(t) => t,
            _ => panic!("Invalid column type {:?}", item),
        };

    };

    (typ, ident, extractor)
}

fn process_fk_field<'a>(
    types: &HashSet<String>,
    field: &Field<'a, String>,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::Ident,
    proc_macro2::TokenStream,
) {
    let Field { name, .. } = field;

    let field_type = Type::NamedType("ID".to_string());
    let typ = process_type(types, &field_type, false);
    let ident = format_ident! {"{}", name.to_lowercase()};

    let extractor = quote! {
        let item = vec.pop().expect("Missing item in row");
        let #ident = match item {
            FtColumn::#typ(t) => t,
            _ => panic!("Invalid column type {:?}", item),
        };

    };

    (typ, ident, extractor)
}

fn process_type_def<'a>(
    query_root: &str,
    namespace: &str,
    types: &HashSet<String>,
    typ: &TypeDefinition<'a, String>,
    processed: &mut HashSet<String>,
    primitives: &HashSet<String>,
) -> Option<proc_macro2::TokenStream> {
    let copy_traits: HashSet<String> =
        HashSet::from_iter(["Jsonb"].iter().map(|x| x.to_string()));
    match typ {
        TypeDefinition::Object(obj) => {
            if obj.name == *query_root {
                return None;
            }

            let name = &obj.name;
            let type_id = type_id(namespace, name);
            let mut block = quote! {};
            let mut row_extractors = quote! {};
            let mut construction = quote! {};
            let mut flattened = quote! {};

            for field in &obj.fields {
                let (mut type_name, mut field_name, mut ext) =
                    process_field(types, field);

                let type_name_str = type_name.to_string();

                if processed.contains(&type_name_str)
                    && !primitives.contains(&type_name_str)
                {
                    (type_name, field_name, ext) = process_fk_field(types, field);
                }

                processed.insert(type_name_str.clone());

                let decoder = if copy_traits.contains(&type_name_str) {
                    quote! { FtColumn::#type_name(self.#field_name.clone()), }
                } else {
                    quote! { FtColumn::#type_name(self.#field_name), }
                };

                block = quote! {
                    #block
                    #field_name: #type_name,
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
            let strct = format_ident! {"{}", name};

            processed.insert(strct.to_string());

            Some(quote! {
                #[derive(Debug, PartialEq, Eq)]
                pub struct #strct {
                    #block
                }

                impl Entity for #strct {
                    const TYPE_ID: u64 = #type_id;

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
        obj => panic!("Unexpected type: {:?}", obj),
    }
}

fn process_definition<'a>(
    query_root: &str,
    namespace: &str,
    types: &HashSet<String>,
    definition: &Definition<'a, String>,
    processed: &mut HashSet<String>,
    primitives: &HashSet<String>,
) -> Option<proc_macro2::TokenStream> {
    match definition {
        Definition::TypeDefinition(def) => {
            process_type_def(query_root, namespace, types, def, processed, primitives)
        }
        Definition::SchemaDefinition(_def) => None,
        def => {
            panic!("Unhandled definition type: {:?}", def);
        }
    }
}

fn get_query_root<'a>(types: &HashSet<String>, ast: &Document<'a, String>) -> String {
    let schema = ast.definitions.iter().find_map(|def| {
        if let Definition::SchemaDefinition(d) = def {
            Some(d)
        } else {
            None
        }
    });

    let SchemaDefinition { query, .. } = schema.expect("Schema definition not found.");

    let name = query
        .as_ref()
        .expect("Schema definition must specify a query root.")
        .into();

    if !types.contains(&name) {
        panic!("Query root not defined.");
    }

    name
}

fn const_item(id: &str, value: &str) -> proc_macro2::TokenStream {
    let ident = format_ident! {"{}", id};

    let fn_ptr = format_ident! {"get_{}_ptr", id.to_lowercase()};
    let fn_len = format_ident! {"get_{}_len", id.to_lowercase()};

    quote! {
        const #ident: &'static str = #value;

        #[no_mangle]
        fn #fn_ptr() -> *const u8 {
            #ident.as_ptr()
        }

        #[no_mangle]
        fn #fn_len() -> u32 {
            #ident.len() as u32
        }
    }
}

pub(crate) fn process_graphql_schema(
    namespace: String,
    schema_name: String,
) -> proc_macro2::TokenStream {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("Manifest dir unknown");

    let mut current = std::path::PathBuf::from(manifest);
    current.push(schema_name);

    let mut file = match File::open(&current) {
        Ok(f) => f,
        Err(e) => {
            proc_macro_error::abort_call_site!(
                "Could not open schema file {:?} {:?}",
                current,
                e
            )
        }
    };

    let mut text = String::new();
    file.read_to_string(&mut text).expect("IO error");

    let base_ast = match parse_schema::<String>(BASE_SCHEMA) {
        Ok(ast) => ast,
        Err(e) => {
            proc_macro_error::abort_call_site!("Error parsing graphql schema {:?}", e)
        }
    };
    let (primitives, _) = get_schema_types(&base_ast);

    let ast = match parse_schema::<String>(&text) {
        Ok(ast) => ast,
        Err(e) => {
            proc_macro_error::abort_call_site!("Error parsing graphql schema {:?}", e)
        }
    };
    let (mut types, _) = get_schema_types(&ast);
    types.extend(primitives.clone());

    let namespace_tokens = const_item("NAMESPACE", &namespace);
    let version = const_item("VERSION", &schema_version(&text));

    let mut output = quote! {
        #namespace_tokens
        #version
    };

    let query_root = get_query_root(&types, &ast);

    let mut processed: HashSet<String> = HashSet::new();

    for definition in ast.definitions.iter() {
        if let Some(def) = process_definition(
            &query_root,
            &namespace,
            &types,
            definition,
            &mut processed,
            &primitives,
        ) {
            output = quote! {
                #output
                #def
            };
        }
    }
    output
}
