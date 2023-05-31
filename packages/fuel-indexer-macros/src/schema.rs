use crate::helpers::*;
use async_graphql_parser::types::{
    BaseType, FieldDefinition, Type, TypeDefinition, TypeKind, TypeSystemDefinition,
};
use async_graphql_value::Name;
use fuel_indexer_database_types::{directives, IdCol};
use fuel_indexer_lib::utils::local_repository_root;
use fuel_indexer_schema::{parser::ParsedGraphQLSchema, utils::*};
use fuel_indexer_types::type_id;
use lazy_static::lazy_static;
use linked_hash_set::LinkedHashSet;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, HashSet};
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

        // Imports for transaction fields.
        // https://github.com/FuelLabs/fuel-indexer/issues/286
        "BlockData",
        "BytecodeLength",
        "BytecodeWitnessIndex",
        "FieldTxPointer",
        "GasLimit",
        "GasPrice",
        "Inputs",
        "Log",
        "LogData",
        "Maturity",
        "MessageId",
        "Outputs",
        "ReceiptsRoot",
        "Script",
        "ScriptData",
        "ScriptResult",
        "StorageSlots",
        "TransactionData",
        "Transfer",
        "TransferOut",
        "TxFieldSalt",
        "TxFieldScript",
        "TxId",
        "Witnesses",
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
            if !schema.has_type(&name) {
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

    let extractor = field_extractor(
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
    Union,
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
        SpecialFieldType::NoRelation | SpecialFieldType::Union => {
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
    let namespace = &schema.namespace;
    let identifier = &schema.identifier;
    match &typ.kind {
        TypeKind::Object(obj) => {
            let object_name = typ.name.to_string();

            let mut fields_map = BTreeMap::new();

            if DISALLOWED_OBJECT_NAMES.contains(object_name.as_str()) {
                panic!("Object name '{object_name}' is reserved.",);
            }

            let type_id =
                type_id(&format!("{namespace}_{identifier}"), object_name.as_str());
            let mut strct_fields = quote! {};
            let mut field_extractors = quote! {};
            let mut from_row = quote! {};
            let mut to_row = quote! {};

            let mut parameters = quote! {};
            let mut hasher = quote! { Sha256::new() };
            let mut construction_for_auto_id = quote! {};

            for field in &obj.fields {
                let field_name = &field.node.name.to_string();
                let field_type = &field.node.ty.node;
                let (mut typ_tokens, mut field_name, mut scalar_typ, mut extractor) =
                    process_field(schema, field_name, field_type);

                let mut field_typ_name = scalar_typ.to_string();

                if schema.is_union_type(&field_typ_name) {
                    (typ_tokens, field_name, scalar_typ, extractor) =
                        process_special_field(
                            schema,
                            &object_name,
                            &field.node,
                            field_type.nullable,
                            SpecialFieldType::Union,
                        );
                    field_typ_name = scalar_typ.to_string();
                }

                if schema.is_possible_foreign_key(&field_typ_name) {
                    (typ_tokens, field_name, scalar_typ, extractor) =
                        process_special_field(
                            schema,
                            &object_name,
                            &field.node,
                            field_type.nullable,
                            SpecialFieldType::ForeignKey,
                        );
                    field_typ_name = scalar_typ.to_string();
                }

                if schema.is_enum_type(&field_typ_name) {
                    (typ_tokens, field_name, scalar_typ, extractor) =
                        process_special_field(
                            schema,
                            &object_name,
                            &field.node,
                            field_type.nullable,
                            SpecialFieldType::Enum,
                        );
                    field_typ_name = scalar_typ.to_string();
                }

                if schema.is_non_indexable_non_enum(&field_typ_name) {
                    (typ_tokens, field_name, scalar_typ, extractor) =
                        process_special_field(
                            schema,
                            &object_name,
                            &field.node,
                            field_type.nullable,
                            SpecialFieldType::NoRelation,
                        );
                    field_typ_name = scalar_typ.to_string();
                }

                fields_map.insert(field_name.to_string(), field_typ_name.clone());

                let is_copy_type = COPY_TYPES.contains(field_typ_name.as_str());
                let is_non_indexable_non_enum =
                    schema.is_non_indexable_non_enum(&object_name);
                let is_copyable = is_copy_type || is_non_indexable_non_enum;

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

                strct_fields = quote! {
                    #strct_fields
                    #field_name: #typ_tokens,
                };

                field_extractors = quote! {
                    #extractor
                    #field_extractors
                };

                from_row = quote! {
                    #from_row
                    #field_name,
                };

                to_row = quote! {
                    #to_row
                    #decoder
                };

                if !INTERNAL_INDEXER_ENTITIES.contains(object_name.as_str())
                    && field_name != "id".to_string()
                {
                    parameters = quote! { #parameters #field_name: #typ_tokens, };

                    if !ASREF_BYTE_TYPES.contains(field_typ_name.as_str()) {
                        hasher = quote! { #hasher.chain_update(#field_name #clone.to_be_bytes())};
                    } else {
                        hasher = quote! { #hasher.chain_update(#field_name #clone)};
                    }

                    construction_for_auto_id = quote! {
                        #construction_for_auto_id
                        #field_name,
                    };
                }
            }

            schema
                .object_field_mappings
                .insert(object_name.clone(), fields_map);
            let strct = format_ident! {"{}", object_name};

            let new_method_impl = generate_struct_new_method_impl(
                strct.clone(),
                parameters,
                hasher,
                object_name,
                construction_for_auto_id,
            );

            let object_trait_impls = generate_object_trait_impls(
                strct,
                strct_fields,
                type_id,
                field_extractors,
                from_row,
                to_row,
                schema.is_native,
            );

            Some(quote! {
                #new_method_impl

                #object_trait_impls
            })
        }
        TypeKind::Enum(e) => {
            let name = typ.name.to_string();
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
        TypeKind::Union(obj) => {
            // We process this type effectively the same as we process `TypeKind::Object`.
            //
            // Except instead of iterating over the object's fields in order to construct the
            // struct, we iterate over the set of all the union's members' fields, and derive
            // the struct from those fields.
            //
            // Same field processing as `TypeKind::Object`, it's just that the source of the fields is different.
            let name = typ.name.to_string();
            let ident = format_ident!("{}", name);

            let type_id = type_id(&format!("{namespace}_{identifier}"), name.as_str());
            let mut strct_fields = quote! {};
            let mut field_extractors = quote! {};
            let mut from_row = quote! {};
            let mut to_row = quote! {};

            obj.members
                .iter()
                .flat_map(|m| {
                    let name = m.node.to_string();
                    schema
                        .object_field_mappings
                        .get(&name)
                        .unwrap_or_else(|| {
                            panic!("Could not find union member '{name}' in the schema.",)
                        })
                        .iter()
                        .map(|(k, v)| (k.to_owned(), v.to_owned()))
                })
                .collect::<LinkedHashSet<(String, String)>>()
                .iter()
                .for_each(|(field_name, field_type_name)| {
                    let field_type = Type {
                        base: BaseType::Named(Name::new(field_type_name)),
                        nullable: field_type_name != IdCol::to_uppercase_str(),
                    };

                    // Since we've already processed the member's fields, we don't need
                    // to do any type of special field processing here.
                    let (typ_tokens, field_name, scalar_typ, extractor) =
                        process_field(schema, field_name, &field_type);

                    let is_copy_type = COPY_TYPES.contains(field_type_name.as_str());
                    let is_non_indexable_non_enum =
                        schema.is_non_indexable_non_enum(&name);
                    let is_copyable = is_copy_type || is_non_indexable_non_enum;

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

                    strct_fields = quote! {
                        #strct_fields
                        #field_name: #typ_tokens,
                    };

                    field_extractors = quote! {
                        #extractor
                        #field_extractors
                    };

                    from_row = quote! {
                        #from_row
                        #field_name,
                    };

                    to_row = quote! {
                        #to_row
                        #decoder
                    };
                });

            Some(generate_object_trait_impls(
                ident,
                strct_fields,
                type_id,
                field_extractors,
                from_row,
                to_row,
                schema.is_native,
            ))
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
