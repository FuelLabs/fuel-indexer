use crate::helpers::{const_item, row_extractor};
use async_graphql_parser::types::{
    BaseType, FieldDefinition, Type, TypeDefinition, TypeKind, TypeSystemDefinition,
};
use fuel_indexer_database_types::directives;
use fuel_indexer_lib::utils::local_repository_root;
use fuel_indexer_schema::{
    parser::ParsedGraphQLSchema,
    utils::{
        get_join_directive_info, get_notable_directive_info,
        inject_native_entities_into_schema, schema_version,
    },
};
use fuel_indexer_types::type_id;
use lazy_static::lazy_static;
use quote::{format_ident, quote};
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

type ProcessedFieldResult = (
    proc_macro2::TokenStream,
    proc_macro2::Ident,
    proc_macro2::Ident,
    proc_macro2::TokenStream,
);

lazy_static! {
    /// Set of types that should be copied instead of referenced.
    static ref COPY_TYPES: HashSet<&'static str> = HashSet::from([
        "Blob",
        "Charfield",
        "HexString",
        "Identity",
        "Json",
        "NoRelation",
        "Option<Blob>",
        "Option<Charfield>",
        "Option<HexString>",
        "Option<Identity>",
        "Option<Json>",
        "Option<NoRelation>",
    ]);

    static ref DISALLOWED_OBJECT_NAMES: HashSet<&'static str> = HashSet::from([
        // Native types.
        "BlockData",
        "HeaderData",
        "Log",
        "LogData",
        "MessageId",
        "Receipt",
        "ScriptResult",
        "Transaction",
        "TransactionData",
        "TransactionStatus",
        "Transfer",
        "TransferOut",

        // Scalars.
        "Address",
        "AssetId",
        "Blob",
        "BlockHeight",
        "BlockId",
        "Boolean",
        "Bytes",
        "Bytes32",
        "Bytes4",
        "Bytes64",
        "Bytes8",
        "Charfield",
        "Color",
        "ContractId",
        "HexString",
        "ID",
        "Identity",
        "Int1",
        "Int16",
        "Int4",
        "Int8",
        "Json",
        "MessageId",
        "Nonce",
        "NoRelation",
        "Salt",
        "Signature",
        "Tai64Timestamp",
        "Timestamp",
        "TxId",
        "UInt1",
        "UInt16",
        "UInt4",
        "UInt8",

        // Temporary types: https://github.com/FuelLabs/fuel-indexer/issues/286
        "Consensus",
        "Genesis",
        "Genesis",
        "PoA",
        "Unknown",

    ]);
}

/// Process a named type into its type tokens, and the Ident for those type tokens.
fn process_type(
    schema: &ParsedGraphQLSchema,
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
    schema: &ParsedGraphQLSchema,
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

    let extractor = row_extractor(
        schema,
        name_ident.clone(),
        typ_ident.clone(),
        field_type.nullable,
    );

    (typ_tokens, name_ident, typ_ident, extractor)
}

/// Type of special fields in GraphQL schema.
enum SpecialFieldType {
    ForeignKey,
    Enum,
    NoRelation,
}

/// Process an object's 'special' field and return a group of tokens.
///
/// Special fields are limited to foreign key and enum fields.
///
/// This is the equivalent of `process_field` but with some pre/post-processing.
fn process_special_field(
    schema: &ParsedGraphQLSchema,
    object_name: &String,
    field: &FieldDefinition,
    is_nullable: bool,
    field_type: SpecialFieldType,
) -> ProcessedFieldResult {
    match field_type {
        SpecialFieldType::ForeignKey => {
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
            let field_type: Type = Type::new(&field_type_name)
                .expect("Could not construct type for processing");

            process_field(schema, &field_name, &field_type)
        }
        SpecialFieldType::Enum => {
            let FieldDefinition {
                name: field_name, ..
            } = field;

            let field_type_name = if !is_nullable {
                ["Charfield".to_string(), "!".to_string()].join("")
            } else {
                "Charfield".to_string()
            };

            let field_type: Type = Type::new(&field_type_name)
                .expect("Could not construct type for processing");

            process_field(schema, &field_name.to_string(), &field_type)
        }
        SpecialFieldType::NoRelation => {
            let FieldDefinition {
                name: field_name, ..
            } = field;

            let field_type_name = if !is_nullable {
                ["NoRelation".to_string(), "!".to_string()].join("")
            } else {
                "NoRelation".to_string()
            };

            let field_type: Type = Type::new(&field_type_name)
                .expect("Could not construct type for processing");

            process_field(schema, &field_name.to_string(), &field_type)
        }
    }
}

/// Process a schema's type definition into the corresponding tokens for use in an indexer module.
fn process_type_def(
    schema: &mut ParsedGraphQLSchema,
    typ: &TypeDefinition,
) -> Option<proc_macro2::TokenStream> {
    match &typ.kind {
        TypeKind::Object(obj) => {
            let object_name = typ.name.to_string();

            if DISALLOWED_OBJECT_NAMES.contains(object_name.as_str()) {
                panic!("Object name '{object_name}' is reserved.",);
            }

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
                    process_field(schema, field_name, field_type);

                let mut field_typ_name = scalar_typ.to_string();

                let directives::NoRelation(is_no_table) =
                    get_notable_directive_info(&field.node).unwrap();

                if is_no_table {
                    schema.non_indexable_type_names.insert(object_name.clone());
                }

                if schema.is_possible_foreign_key(&field_typ_name) {
                    (typ_tokens, field_name, scalar_typ, ext) = process_special_field(
                        schema,
                        &object_name,
                        &field.node,
                        field_type.nullable,
                        SpecialFieldType::ForeignKey,
                    );
                    field_typ_name = scalar_typ.to_string();
                }

                if schema.is_enum_type(&field_typ_name) {
                    (typ_tokens, field_name, scalar_typ, ext) = process_special_field(
                        schema,
                        &object_name,
                        &field.node,
                        field_type.nullable,
                        SpecialFieldType::Enum,
                    );
                    field_typ_name = scalar_typ.to_string();
                }

                if schema.is_non_indexable_non_enum(&field_typ_name) {
                    (typ_tokens, field_name, scalar_typ, ext) = process_special_field(
                        schema,
                        &object_name,
                        &field.node,
                        field_type.nullable,
                        SpecialFieldType::NoRelation,
                    );
                    field_typ_name = scalar_typ.to_string();
                }

                schema.parsed_type_names.insert(field_typ_name.clone());
                let is_copyable = COPY_TYPES.contains(field_typ_name.as_str())
                    || schema.is_non_indexable_non_enum(&object_name);

                let clone = if is_copyable {
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
                    #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

                    impl From<#strct> for Json {
                        fn from(value: #strct) -> Self {
                            let s = serde_json::to_string(&value).expect("Serde error.");
                            Self(s)
                        }
                    }

                    impl From<Json> for #strct {
                        fn from(value: Json) -> Self {
                            let s: #strct = serde_json::from_str(&value.0).expect("Serde error.");
                            s
                        }
                    }
                })
            } else {
                // TODO: derive Default here https://github.com/FuelLabs/fuels-rs/pull/977
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

                    impl From<#strct> for Json {
                        fn from(value: #strct) -> Self {
                            let s = serde_json::to_string(&value).expect("Serde error.");
                            Self(s)
                        }
                    }

                    impl From<Json> for #strct {
                        fn from(value: Json) -> Self {
                            let s: #strct = serde_json::from_str(&value.0).expect("Serde error.");
                            s
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

            let to_enum = e
                .values
                .iter()
                .map(|v| {
                    let ident = format_ident! {"{}", v.node.value.to_string()};
                    let as_str = format!("{}::{}", name, ident);
                    quote! { #as_str => #name::#ident, }
                })
                .collect::<Vec<proc_macro2::TokenStream>>();

            let from_enum = e
                .values
                .iter()
                .map(|v| {
                    let ident = format_ident! {"{}", v.node.value.to_string()};
                    let as_str = format!("{}::{}", name, ident);
                    quote! { #name::#ident => #as_str.to_string(), }
                })
                .collect::<Vec<proc_macro2::TokenStream>>();

            // TODO: derive Default here https://github.com/FuelLabs/fuels-rs/pull/977
            Some(quote! {
                #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
                pub enum #name {
                    #(#values),*
                }

                impl From<#name> for String {
                    fn from(val: #name) -> Self {
                        match val {
                            #(#from_enum)*
                            _ => panic!("Unrecognized enum value."),
                        }
                    }
                }

                impl From<String> for #name {
                    fn from(val: String) -> Self {
                        match val.as_ref() {
                            #(#to_enum)*
                            _ => panic!("Unrecognized enum value."),
                        }
                    }
                }
            })
        }
        _ => panic!("Unexpected type: '{:?}'", typ.kind),
    }
}

/// Process a schema definition into the corresponding tokens for use in an indexer module.
fn process_definition(
    schema: &mut ParsedGraphQLSchema,
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
    let namespace_tokens = const_item("NAMESPACE", &namespace);
    let identifer_tokens = const_item("IDENTIFIER", &identifier);
    let version_tokens = const_item("VERSION", &schema_version(&text));

    let mut output = quote! {
        #namespace_tokens
        #identifer_tokens
        #version_tokens
    };

    let mut schema =
        ParsedGraphQLSchema::new(&namespace, &identifier, is_native, Some(&text))
            .expect("Bad schema.");

    for definition in schema.ast.clone().definitions.iter() {
        if let Some(def) = process_definition(&mut schema, definition) {
            output = quote! {
                #output
                #def
            };
        }
    }
    output
}
