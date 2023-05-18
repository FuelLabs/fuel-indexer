use crate::helpers::{const_item, row_extractor};
use async_graphql_parser::parse_schema;
use async_graphql_parser::types::{
    BaseType, FieldDefinition, ServiceDocument, Type, TypeDefinition, TypeKind,
    TypeSystemDefinition,
};
use fuel_indexer_database_types::directives;
use fuel_indexer_lib::utils::local_repository_root;
use fuel_indexer_schema::utils::{
    build_schema_fields_and_types_map, build_schema_types_set, get_join_directive_info,
    inject_native_entities_into_schema, schema_version, BASE_SCHEMA,
};
use fuel_indexer_types::type_id;
use lazy_static::lazy_static;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Set of types that should be copied instead of referenced.
lazy_static! {
    static ref COPY_TYPES: HashSet<&'static str> = HashSet::from([
        "Json",
        "Charfield",
        "Identity",
        "Blob",
        "HexString",
        "Option<Json>",
        "Option<Blob>",
        "Option<Charfield>",
        "Option<Identity>",
        "Option<HexString>",
    ]);
}

/// A wrapper object used to encapsulate a lot of the boilerplate logic related
/// to parsing schema, creating mappings of types, fields, objects, etc.
struct Schema {
    /// Namespace of the indexer.
    namespace: String,

    /// Identifier of the indexer.
    identifier: String,

    /// Whether we're building schema for native execution.
    is_native: bool,

    /// All unique names of types in the schema (whether objects, enums, or scalars).
    type_names: HashSet<String>,

    /// All unique names of enums in the schema.
    enum_names: HashSet<String>,

    /// All unique names of types for which tables should _not_ be created.
    non_indexable_type_names: HashSet<String>,

    /// All unique names of types that have already been parsed.
    parsed_type_names: HashSet<String>,

    /// All unique names of types that are possible foreign keys.
    foreign_key_names: HashSet<String>,

    /// A mapping of fully qualitified field names to their field types.
    field_type_mappings: HashMap<String, String>,

    /// All unique names of scalar types in the schema.
    scalar_names: HashSet<String>,
}

impl Schema {
    /// Create a new Schema.
    pub fn new(
        namespace: &str,
        identifier: &str,
        is_native: bool,
        ast: &ServiceDocument,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let base_ast = match parse_schema(BASE_SCHEMA) {
            Ok(ast) => ast,
            Err(e) => {
                proc_macro_error::abort_call_site!("Error parsing graphql schema {:?}", e)
            }
        };

        let (mut type_names, _) = build_schema_types_set(&ast);
        let (scalar_names, _) = build_schema_types_set(&base_ast);
        type_names.extend(scalar_names.clone());

        Ok(Self {
            namespace: namespace.to_string(),
            identifier: identifier.to_string(),
            is_native,
            type_names,
            enum_names: HashSet::new(),
            non_indexable_type_names: HashSet::new(),
            parsed_type_names: HashSet::new(),
            foreign_key_names: HashSet::new(),
            field_type_mappings: build_schema_fields_and_types_map(&ast)?,
            scalar_names,
        })
    }

    /// Whether the schema has a scalar type with the given name.
    pub fn has_scalar(&self, name: &str) -> bool {
        self.scalar_names.contains(name)
    }

    /// Whether the given field type name is a possible foreign key.
    pub fn is_possible_foreign_key(&self, name: &str) -> bool {
        self.parsed_type_names.contains(name) && !self.has_scalar(name)
    }
}

/// Process a named type into its type tokens, and the Ident for those type tokens.
fn process_type(
    schema: &Schema,
    typ: &Type,
) -> (proc_macro2::TokenStream, proc_macro2::Ident) {
    match &typ.base {
        BaseType::Named(t) => {
            let name = t.to_string();
            if !schema.type_names.contains(&name) {
                panic!("Type '{name}' is not defined in the schema.",);
            }

            let name = format_ident! {"{}", name};

            if typ.nullable {
                (quote! { Option<#name> }, name)
            } else {
                (quote! { #name }, name)
            }
        }
        BaseType::List(_t) => panic!("Got a list type, we don't handle this yet..."),
    }
}

/// Process an object's field and return a group of tokens.
///
/// This group of tokens include:
///     - The field's type tokens.
///     - The field's name as an Ident.
///     - The field's type as an Ident.
///     - The field's row extractor tokens.
fn process_field(
    schema: &Schema,
    field_name: &String,
    field_type: &Type,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::Ident,
    proc_macro2::Ident,
    proc_macro2::TokenStream,
) {
    let (typ_tokens, typ_ident) = process_type(schema, field_type);
    let name_ident = format_ident! {"{}", field_name.to_string()};

    let extractor =
        row_extractor(name_ident.clone(), typ_ident.clone(), field_type.nullable);

    (typ_tokens, name_ident, typ_ident, extractor)
}

/// Process an object's foreign key field and return a group of tokens.
///
/// This group of tokens include:
///     - The field's type tokens.
///     - The field's name as an Ident.
///     - The field's type as an Ident.
///     - The field's row extractor tokens.
///
/// This is the equivalent of `process_field` but with some pre-processing.
fn process_fk_field(
    schema: &Schema,
    object_name: &String,
    field: &FieldDefinition,
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
    } = get_join_directive_info(field, object_name, &schema.field_type_mappings);

    let field_type_name = if !is_nullable {
        [reference_field_type_name, "!".to_string()].join("")
    } else {
        reference_field_type_name
    };
    let field_type: Type =
        Type::new(&field_type_name).expect("Could not construct type for processing");

    process_field(schema, &field_name, &field_type)
}

/// Process a schema's type definition into the corresponding tokens for use in an indexer module.
fn process_type_def(
    schema: &mut Schema,
    typ: &TypeDefinition,
) -> Option<proc_macro2::TokenStream> {
    match &typ.kind {
        TypeKind::Object(obj) => {
            let object_name = typ.name.to_string();

            let namespace = &schema.namespace;
            let identifier = &schema.identifier;

            let type_id =
                type_id(&format!("{namespace}_{identifier}"), object_name.as_str());
            let mut block = quote! {};
            let mut row_extractors = quote! {};
            let mut construction = quote! {};
            let mut flattened = quote! {};

            for field in &obj.fields {
                let field_name = &field.node.name.to_string();
                let field_type = &field.node.ty.node;
                let (mut typ_tokens, mut field_name, mut scalar_typ, mut ext) =
                    process_field(&schema, field_name, field_type);

                let mut column_type_name = scalar_typ.to_string();

                if schema.is_possible_foreign_key(&column_type_name) {
                    (typ_tokens, field_name, scalar_typ, ext) = process_fk_field(
                        &schema,
                        &object_name,
                        &field.node,
                        field_type.nullable,
                    );
                    column_type_name = scalar_typ.to_string();
                }

                schema.parsed_type_names.insert(column_type_name.clone());

                if schema.non_indexable_type_names.contains(&column_type_name) {
                    if schema.enum_names.contains(&column_type_name) {
                        scalar_typ = format_ident!("{}", "Enum");
                        column_type_name = "u8".to_string();
                    } else {
                        continue;
                    }
                }

                let clone = if COPY_TYPES.contains(column_type_name.as_str()) {
                    quote! {.clone()}
                } else {
                    quote! {}
                };

                let decoder = if field_type.nullable {
                    quote! { FtColumn::#scalar_typ(self.#field_name #clone), }
                } else {
                    quote! { FtColumn::#scalar_typ(Some(self.#field_name #clone)), }
                };

                block = quote! {
                    #block
                    #field_name: #typ_tokens,
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

            schema.parsed_type_names.insert(strct.to_string());

            if schema.is_native {
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
                    #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
        }
        TypeKind::Enum(e) => {
            let name = typ.name.to_string();
            schema.non_indexable_type_names.insert(name.clone());
            schema.enum_names.insert(name.clone());

            let name = format_ident!("{}", name);

            let values = e.values.iter().map(|v| {
                let ident = format_ident! {"{}", v.node.value.to_string()};
                quote! { #ident }
            });
            Some(quote! {
                #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
                pub enum #name {
                    #(#values),*
                }
            })
        }
        _ => panic!("Unexpected type: '{:?}'", typ.kind),
    }
}

fn process_definition(
    schema: &mut Schema,
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

    let ast = match parse_schema(&text) {
        Ok(ast) => ast,
        Err(e) => {
            proc_macro_error::abort_call_site!("Error parsing graphql schema {:?}", e)
        }
    };

    let namespace_tokens = const_item("NAMESPACE", &namespace);
    let identifer_tokens = const_item("IDENTIFIER", &identifier);
    let version_tokens = const_item("VERSION", &schema_version(&text));

    let mut output = quote! {
        #namespace_tokens
        #identifer_tokens
        #version_tokens
    };

    let mut schema =
        Schema::new(&namespace, &identifier, is_native, &ast).expect("Bad schema.");

    for definition in ast.definitions.iter() {
        if let Some(def) = process_definition(&mut schema, definition) {
            output = quote! {
                #output
                #def
            };
        }
    }
    output
}
