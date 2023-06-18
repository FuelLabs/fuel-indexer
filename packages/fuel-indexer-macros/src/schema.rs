use crate::{
    constants::*, decoder::Decoder, helpers::*, validator::GraphQLSchemaValidator,
};
use async_graphql_parser::types::{
    BaseType, FieldDefinition, Type, TypeDefinition, TypeKind, TypeSystemDefinition,
};
use async_graphql_value::Name;
use fuel_indexer_database_types::*;
use fuel_indexer_lib::utils::local_repository_root;
use fuel_indexer_schema::{parser::ParsedGraphQLSchema, utils::*};
use fuel_indexer_types::{graphql::GraphQLSchema, type_id};
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

// REFACTOR: remove
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
    field_type: Type,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::Ident,
    proc_macro2::Ident,
    proc_macro2::TokenStream,
) {
    let (typ_tokens, typ_ident) = process_type(schema, &field_type);
    let name_ident = format_ident! {"{}", field_name.to_string()};

    let extractor = field_extractor(
        schema,
        name_ident.clone(),
        typ_ident.clone(),
        field_type.nullable,
    );

    (typ_tokens, name_ident, typ_ident, extractor)
}

// TODO: combine process_field and process_special_field
//
// processing FKs and virtuals should be recursive:
//
// process_field(virtual) -> process_field(json)
// process_field(fk) -> process_field(join_id)
// process_field(regular) -> .. | process_field(virtual) | process_field(union) | process_field(enum)
// process_field(union) -> (all union field members are already processed)
// process_field(enum) -> process_field(string)

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
    kind: FieldKind,
) -> ProcessedFieldResult {
    let FieldDefinition {
        name: field_name, ..
    } = field.clone();

    match kind {
        FieldKind::ForeignKey => {
            let directives::Join {
                field_name,
                reference_field_type_name,
                ..
            } = get_join_directive_info(field, object_name, &schema.field_type_mappings);

            let field_typ_name = if !is_nullable {
                [reference_field_type_name, "!".to_string()].join("")
            } else {
                reference_field_type_name
            };
            // let field_type: Type = Type::new(&field_typ_name)
            //     .expect("Could not construct type for processing");

            process_field(schema, &field_name, field.ty.node.clone())
        }
        FieldKind::Enum => {
            // let field_typ_name = if !is_nullable {
            //     ["Charfield".to_string(), "!".to_string()].join("")
            // } else {
            //     "Charfield".to_string()
            // };

            // let field_type: Type = Type::new(&field_typ_name)
            //     .expect("Could not construct type for processing");

            let ty =
                Type::new("Charfield").expect("Could not construct type for processing.");
            process_field(schema, &field_name.to_string(), ty)
        }
        FieldKind::Virtual => {
            // let field_typ_name = if !is_nullable {
            //     ["Virtual".to_string(), "!".to_string()].join("")
            // } else {
            //     "Virtual".to_string()
            // };

            // let field_type: Type = Type::new(&field_typ_name)
            //     .expect("Could not construct type for processing");

            let ty =
                Type::new("Virtual").expect("Could not construct type for processing.");
            process_field(schema, &field_name.to_string(), ty)
        }
        FieldKind::Union => {
            let field_typ_name = field.ty.to_string();
            match schema.is_non_indexable_non_enum(&field_typ_name) {
                true => process_special_field(
                    schema,
                    object_name,
                    field,
                    is_nullable,
                    FieldKind::Virtual,
                ),
                false => process_special_field(
                    schema,
                    object_name,
                    field,
                    is_nullable,
                    FieldKind::ForeignKey,
                ),
            }
        }
        FieldKind::Regular | FieldKind::Unknown => {
            process_field(schema, &field.name.to_string(), field.ty.node.clone())
        }
    }
}

fn foo_process_type_def(
    parser: &ParsedGraphQLSchema,
    typ: &TypeDefinition,
) -> Option<proc_macro2::TokenStream> {
    let namespace = &parser.namespace;
    let identifier = &parser.identifier;
    let typekind_name = typ.name.to_string();
    let decoder = match &typ.kind {
        TypeKind::Object(o) => Decoder::from_object(typekind_name, o.to_owned(), parser),
        TypeKind::Enum(e) => Decoder::from_enum(typekind_name, e.to_owned(), parser),
        TypeKind::Union(u) => Decoder::from_union(typekind_name, u.to_owned(), parser),
        _ => proc_macro_error::abort_call_site!(
            "Unrecognized TypeKind in GraphQL schema: {:?}",
            typ.kind
        ),
    };

    Some(decoder.into())
}

// REFACTOR: remove
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
            let mut field_construction_for_new_impl = quote! {};

            let fields = obj
                .fields
                .iter()
                .map(|f| f.node.name.node.to_string())
                .collect::<Vec<String>>();
            let field_set: HashSet<String> =
                HashSet::from_iter(fields.iter().map(|f| f.to_owned()));

            for field in &obj.fields {
                let field_name = &field.node.name.to_string();
                let field_type = &field.node.ty.node;
                let (mut typ_tokens, mut field_name, mut scalar_typ, mut extractor) =
                    process_field(schema, field_name, field_type.clone());

                let mut field_typ_name = scalar_typ.to_string();

                if schema.is_union_type(&field_typ_name) {
                    (typ_tokens, field_name, scalar_typ, extractor) =
                        process_special_field(
                            schema,
                            &object_name,
                            &field.node,
                            field_type.nullable,
                            FieldKind::Union,
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
                            FieldKind::ForeignKey,
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
                            FieldKind::Enum,
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
                            FieldKind::Virtual,
                        );
                    field_typ_name = scalar_typ.to_string();
                }

                fields_map.insert(field_name.to_string(), field_typ_name.clone());

                let is_copy_type = COPY_TYPES.contains(field_typ_name.as_str());
                let is_non_indexable_non_enum =
                    schema.is_non_indexable_non_enum(&object_name);
                let is_copyable = is_copy_type || is_non_indexable_non_enum;

                let clone = clone_tokens(field_typ_name.as_str());

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

                let unwrap_or_def = if field_type.nullable {
                    if EXTERNAL_FIELD_TYPES.contains(field_typ_name.as_str()) {
                        unwrap_or_default_for_external_type(&field_typ_name)
                    } else {
                        quote! { .unwrap_or_default() }
                    }
                } else {
                    quote! {}
                };

                let to_bytes = if EXTERNAL_FIELD_TYPES.contains(field_typ_name.as_str()) {
                    to_bytes_method_for_external_type(&field_typ_name)
                } else if !ASREF_BYTE_TYPES.contains(field_typ_name.as_str()) {
                    quote! { .to_le_bytes() }
                } else {
                    quote! {}
                };

                if field_set.contains(&IdCol::to_lowercase_string())
                    && !INTERNAL_INDEXER_ENTITIES.contains(object_name.as_str())
                    && field_name != IdCol::to_lowercase_string()
                {
                    parameters = quote! { #parameters #field_name: #typ_tokens, };

                    if !NONDIGESTIBLE_FIELD_TYPES.contains(field_typ_name.as_str()) {
                        hasher = quote! { #hasher.chain_update(#field_name #clone #unwrap_or_def #to_bytes)};
                    }

                    field_construction_for_new_impl = quote! {
                        #field_construction_for_new_impl
                        #field_name,
                    };
                }
            }

            schema
                .object_field_mappings
                .insert(object_name.clone(), fields_map);
            let strct = format_ident! {"{}", object_name};

            let object_trait_impls = generate_object_trait_impls(
                strct.clone(),
                strct_fields,
                type_id,
                field_extractors,
                from_row,
                to_row,
                schema.is_native,
                ImplNewParameters::ObjectType {
                    strct,
                    parameters,
                    hasher,
                    object_name,
                    struct_fields: field_construction_for_new_impl,
                    is_native: schema.is_native,
                    field_set,
                },
            );

            Some(quote! {
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

            let mut derived_type_fields = HashSet::new();
            let mut union_field_set = HashSet::new();

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
                .for_each(|(field_name, field_typ_name)| {

                    // Field types must be consistent across all members of a union.
                    if derived_type_fields.contains(field_name) {
                        panic!("Derived type from Union({}) contains Field({}) which does not have a consistent type across all members.", name, field_name);
                    }

                    derived_type_fields.insert(field_name);

                    // We can't derive this FieldType info from the actual field because since
                    // this is a union, all fields are nullable except field type `ID` (if present).
                    let field_type = Type {
                        base: BaseType::Named(Name::new(field_typ_name)),
                        nullable: field_typ_name != IdCol::to_uppercase_str(),
                    };

                    union_field_set.insert(field_name.clone());

                    // Since we've already processed the member's fields, we don't need
                    // to do any type of special field processing here.
                    let (typ_tokens, field_name, scalar_typ, extractor) =
                        process_field(schema, field_name, field_type.clone());

                    let is_copy_type = COPY_TYPES.contains(field_typ_name.as_str());
                    let is_non_indexable_non_enum =
                        schema.is_non_indexable_non_enum(&name);
                    let is_copyable = is_copy_type || is_non_indexable_non_enum;

                    let clone = if is_copyable {
                        quote! {.clone()}
                    } else {
                        quote! {}
                    };

                    // For union types, all field types except `ID` (if present) are nullable.
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

            let object_trait_impls = generate_object_trait_impls(
                ident.clone(),
                strct_fields,
                type_id,
                field_extractors,
                from_row,
                to_row,
                schema.is_native,
                ImplNewParameters::UnionType {
                    schema: schema.clone(),
                    union_obj: obj.clone(),
                    union_ident: ident,
                    union_field_set,
                },
            );

            Some(quote! {
                #object_trait_impls
            })
        }
        _ => proc_macro_error::abort_call_site!(
            "Unrecognized TypeKind in GraphQL schema: {:?}",
            typ.kind
        ),
    }
}

/// Process a schema definition into the corresponding tokens for use in an indexer module.
fn process_definition(
    schema: &ParsedGraphQLSchema,
    definition: &TypeSystemDefinition,
) -> Option<proc_macro2::TokenStream> {
    match definition {
        TypeSystemDefinition::Type(def) => foo_process_type_def(schema, &def.node),
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
    let namespace_tokens = const_item("NAMESPACE", &namespace);
    let identifer_tokens = const_item("IDENTIFIER", &identifier);

    let path = local_repository_root()
        .map(|p| Path::new(&p).join(schema_path.clone()))
        .unwrap_or_else(|| PathBuf::from(&schema_path));

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

    let version_tokens = const_item("VERSION", &schema.version());

    let mut output = quote! {
        #namespace_tokens
        #identifer_tokens
        #version_tokens
    };

    let schema =
        ParsedGraphQLSchema::new(&namespace, &identifier, is_native, Some(&schema))
            .expect("Failed to parse GraphQL schema.");

    for definition in schema.ast.clone().definitions.iter() {
        if let Some(def) = process_definition(&schema, definition) {
            output = quote! {
                #output
                #def
            };
        }
    }
    output
}
