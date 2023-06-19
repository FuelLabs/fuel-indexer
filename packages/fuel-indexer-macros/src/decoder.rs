use crate::validator::GraphQLSchemaValidator;
use crate::{constants::*, helpers::*};
use async_graphql_parser::types::{
    BaseType, EnumType, FieldDefinition, ObjectType, ServiceDocument, Type,
    TypeDefinition, TypeKind, TypeSystemDefinition, UnionType,
};
use async_graphql_parser::{Pos, Positioned};
use async_graphql_value::Name;
use fuel_indexer_database_types::IdCol;
use fuel_indexer_schema::parser::ParsedGraphQLSchema;
use linked_hash_set::LinkedHashSet;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, HashSet};
use syn::Ident;

pub enum ExecutionSource {
    Native,
    Wasm,
}

pub trait Decoder {
    /// Create a decoder from a GraphQL `TypeDefinition`.
    fn from_typedef(typ: &TypeDefinition, parsed: &ParsedGraphQLSchema) -> Self;
}

/// A wrapper object used to process GraphQL `TypeKind::Object` type definitions
/// into a format from which Rust tokens can be generated.
pub struct ObjectDecoder {
    /// The name of the GraphQL object (as a `syn::Ident`).
    ident: Ident,

    /// Tokens used to create fields in the struct definition.
    struct_fields: TokenStream,

    /// Tokens used to extract each individual field from a row.
    field_extractors: TokenStream,

    /// Tokens used to create fields in the `Entity::from_row` function.
    from_row: TokenStream,

    /// Tokens used to create fields in the `Entity::to_row` function.
    to_row: TokenStream,

    /// Tokens used to create fields in the `Entity::new` function.
    parameters: TokenStream,

    /// Tokens used to derive a unique ID for this object (if specified).
    hasher: TokenStream,

    /// Tokens used to create fields in the `Entity::new` function.
    impl_new_fields: TokenStream,

    /// Tokens for the parameters of the `Entity::new` function.
    impl_new_params: ImplNewParameters,

    /// The source of the GraphQL schema.
    source: ExecutionSource,

    /// The unique ID of this GraphQL type.
    type_id: i64,
}

impl Default for ObjectDecoder {
    fn default() -> Self {
        Self {
            ident: format_ident!("Decoder"),
            struct_fields: quote! {},
            field_extractors: quote! {},
            from_row: quote! {},
            to_row: quote! {},
            parameters: quote! {},
            hasher: quote! { Sha256::new() },
            impl_new_fields: quote! {},
            source: ExecutionSource::Wasm,
            impl_new_params: ImplNewParameters::ObjectType {
                strct: format_ident!("Decoder"),
                parameters: quote! {},
                hasher: quote! {},
                object_name: "".to_string(),
                struct_fields: quote! {},
                is_native: false,
                field_set: HashSet::new(),
            },
            type_id: 0,
        }
    }
}

impl Decoder for ObjectDecoder {
    /// Create a decoder from a GraphQL `TypeKind::Object`.
    fn from_typedef(typ: &TypeDefinition, parsed: &ParsedGraphQLSchema) -> Self {
        match &typ.kind {
            TypeKind::Object(o) => {
                let obj_name = typ.name.to_string();
                let ident = format_ident!("{}", obj_name);
                let type_id =
                    typedef_type_id(&parsed.namespace, &parsed.identifier, &obj_name);

                let mut struct_fields = quote! {};
                let mut field_extractors = quote! {};
                let mut from_row = quote! {};
                let mut to_row = quote! {};
                let mut parameters = quote! {};
                let mut hasher = quote! { Sha256::new() };
                let mut impl_new_fields = quote! {};

                let mut fields_map = BTreeMap::new();
                let obj_fields = parsed
                    .object_field_mappings
                    .get(&obj_name)
                    .expect("")
                    .iter()
                    .map(|(k, v)| k.to_owned())
                    .collect::<HashSet<String>>();
                GraphQLSchemaValidator::check_disallowed_graphql_typedef_name(&obj_name);

                for field in &o.fields {
                    let (typ_tokens, field_name, field_type_scalar_name, extractor) =
                        process_typedef_field(parsed, field.node.clone(), &obj_name);

                    fields_map.insert(
                        field_name.to_string(),
                        field_type_scalar_name.to_string(),
                    );

                    let clone = clone_tokens(&field_type_scalar_name.to_string());
                    let field_decoder = field_decoder_tokens(
                        field.node.ty.node.nullable,
                        &field_type_scalar_name.to_string(),
                        &field_name,
                        clone.clone(),
                    );

                    struct_fields = quote! {
                        #struct_fields
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
                        #field_decoder
                    };

                    let unwrap_or_default = unwrap_or_default_tokens(
                        &field_type_scalar_name.to_string(),
                        field.node.ty.node.nullable,
                    );
                    let to_bytes = to_bytes_tokens(&field_type_scalar_name.to_string());

                    if can_derive_id(&obj_fields, &field_name.to_string(), &obj_name) {
                        parameters =
                            parameters_tokens(parameters, &field_name, typ_tokens);
                        if let Some(tokens) = hasher_tokens(
                            &field_type_scalar_name.to_string(),
                            hasher.clone(),
                            &field_name,
                            clone,
                            unwrap_or_default,
                            to_bytes,
                        ) {
                            hasher = tokens;
                        }

                        impl_new_fields = quote! {
                            #impl_new_fields
                            #field_name,
                        };
                    }
                }

                Self {
                    ident: ident.clone(),
                    struct_fields,
                    field_extractors,
                    from_row,
                    to_row,
                    parameters: parameters.clone(),
                    hasher: hasher.clone(),
                    impl_new_fields: impl_new_fields.clone(),
                    source: ExecutionSource::Wasm,
                    impl_new_params: ImplNewParameters::ObjectType {
                        // standardize all these names
                        strct: ident,
                        parameters,
                        hasher,
                        object_name: obj_name,
                        struct_fields: impl_new_fields,
                        is_native: parsed.is_native,
                        field_set: obj_fields,
                    },
                    type_id,
                }
            }
            TypeKind::Union(u) => {
                let union_name = typ.name.to_string();
                let ident = format_ident!("{}", union_name);
                let type_id =
                    typedef_type_id(&parsed.namespace, &parsed.identifier, &union_name);

                let mut struct_fields = quote! {};
                let mut field_extractors = quote! {};
                let mut from_row = quote! {};
                let mut to_row = quote! {};
                let mut parameters = quote! {};
                let mut hasher = quote! { Sha256::new() };
                let mut impl_new_fields = quote! {};

                let mut derived_type_fields = HashSet::new();
                let mut union_field_set = HashSet::new();

                u.members
                .iter()
                .flat_map(|m| {
                    let name = m.node.to_string();
                    parsed
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
                        panic!("Derived type from Union({}) contains Field({}) which does not have a consistent type across all members.", union_name, field_name);
                    }

                    derived_type_fields.insert(field_name);

                    let field = FieldDefinition {
                        description: None,
                        name: Positioned::new(Name::new(field_name), Pos::default()),
                        arguments: Vec::new(),
                        ty: Positioned::new(
                            Type {
                                base: BaseType::Named(Name::new(field_typ_name)),
                                nullable: field_typ_name != IdCol::to_uppercase_str(),
                            },
                            Pos::default(),
                        ),
                        directives: Vec::new(),
                    };


                    union_field_set.insert(field_name.clone());

                    // Since we've already processed the member's fields, we don't need
                    // to do any type of special field processing here.
                    let (typ_tokens, field_name, field_type_scalar_name, extractor) =
                        process_typedef_field(parsed, field.clone(), &union_name);

                    let clone = clone_tokens(&field_type_scalar_name.to_string());
                    let field_decoder = field_decoder_tokens(
                        field.ty.node.nullable,
                        &field_type_scalar_name.to_string(),
                        &field_name,
                        clone.clone(),
                    );

                    struct_fields = quote! {
                        #struct_fields
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
                        #field_decoder
                    };
                });

                Self {
                    ident,
                    type_id,
                    struct_fields,
                    field_extractors,
                    from_row,
                    to_row,
                    ..Self::default()
                }
            }
            _ => panic!("Expected TypeKind::Object or TypeKind::Union."),
        }
    }
}

/// A wrapper object used to process GraphQL `TypeKind::Enum` type definitions
/// into a format from which Rust tokens can be generated.
pub struct EnumDecoder {
    /// The name of the GraphQL enum (as a `syn::Ident`).
    ident: Ident,

    /// Tokens used to create fields in the `From<String> for #ident` function.
    to_enum: Vec<proc_macro2::TokenStream>,

    /// Tokens used to create fields in the `From<#ident> for String` function.
    from_enum: Vec<proc_macro2::TokenStream>,

    /// Tokens used to create fields in the enum definition.
    values: Vec<TokenStream>,

    /// The unique ID of this GraphQL type.
    type_id: i64,
}

impl Decoder for EnumDecoder {
    /// Create a decoder from a GraphQL `TypeKind::Enum`.
    fn from_typedef(typ: &TypeDefinition, parsed: &ParsedGraphQLSchema) -> Self {
        match &typ.kind {
            TypeKind::Enum(e) => {
                let enum_name = typ.name.to_string();
                let ident = format_ident!("{}", enum_name);
                let type_id =
                    typedef_type_id(&parsed.namespace, &parsed.identifier, &enum_name);

                let values = e
                    .values
                    .iter()
                    .map(|v| {
                        let ident = format_ident! {"{}", v.node.value.to_string()};
                        quote! { #ident }
                    })
                    .collect::<Vec<TokenStream>>();

                let to_enum = e
                    .values
                    .iter()
                    .map(|v| {
                        let value_ident = format_ident! {"{}", v.node.value.to_string()};
                        let as_str = format!("{}::{}", ident, value_ident);
                        quote! { #as_str => #ident::#value_ident, }
                    })
                    .collect::<Vec<proc_macro2::TokenStream>>();

                let from_enum = e
                    .values
                    .iter()
                    .map(|v| {
                        let value_ident = format_ident! {"{}", v.node.value.to_string()};
                        let as_str = format!("{}::{}", ident, value_ident);
                        quote! { #ident::#value_ident => #as_str.to_string(), }
                    })
                    .collect::<Vec<proc_macro2::TokenStream>>();

                Self {
                    ident,
                    to_enum,
                    from_enum,
                    values,
                    type_id,
                }
            }
            _ => panic!("Expected TypeKind::Enum"),
        }
    }
}

impl From<ObjectDecoder> for TokenStream {
    fn from(decoder: ObjectDecoder) -> Self {
        let ObjectDecoder {
            struct_fields,
            ident,
            field_extractors,
            from_row,
            to_row,
            parameters,
            hasher,
            impl_new_fields,
            impl_new_params,
            source,
            type_id,
            ..
        } = decoder;

        let impl_json = quote! {

            impl From<#ident> for Json {
                fn from(value: #ident) -> Self {
                    let s = serde_json::to_string(&value).expect("Serde error.");
                    Self(s)
                }
            }

            impl From<Json> for #ident {
                fn from(value: Json) -> Self {
                    let s: #ident = serde_json::from_str(&value.0).expect("Serde error.");
                    s
                }
            }
        };

        let impl_entity = match source {
            ExecutionSource::Native => quote! {
                #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
                pub struct #ident {
                    #struct_fields
                }

                #[async_trait::async_trait]
                impl Entity for #ident {
                    const TYPE_ID: i64 = #type_id;

                    fn from_row(mut vec: Vec<FtColumn>) -> Self {
                        #field_extractors
                        Self {
                            #from_row
                        }
                    }

                    fn to_row(&self) -> Vec<FtColumn> {
                        vec![
                            #to_row
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
            },
            ExecutionSource::Wasm => quote! {
                #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
                pub struct #ident {
                    #struct_fields
                }

                impl Entity for #ident {
                    const TYPE_ID: i64 = #type_id;

                    fn from_row(mut vec: Vec<FtColumn>) -> Self {
                        #field_extractors
                        Self {
                            #from_row
                        }
                    }

                    fn to_row(&self) -> Vec<FtColumn> {
                        vec![
                            #to_row
                        ]
                    }

                }

            },
        };

        let impl_new = match impl_new_params {
            ImplNewParameters::ObjectType {
                strct,
                parameters,
                hasher,
                object_name,
                struct_fields,
                is_native,
                field_set,
            } => {
                if field_set.contains(&IdCol::to_lowercase_string()) {
                    generate_struct_new_method_impl(
                        strct,
                        parameters,
                        hasher,
                        object_name,
                        struct_fields,
                        is_native,
                    )
                } else {
                    quote! {}
                }
            }
            ImplNewParameters::UnionType {
                schema,
                union_obj,
                union_ident,
                union_field_set,
            } => generate_from_traits_for_union(
                &schema,
                &union_obj,
                union_ident,
                union_field_set,
            ),
        };

        quote! {
            #impl_entity

            #impl_new

            #impl_json
        }
    }
}

impl From<EnumDecoder> for TokenStream {
    fn from(decoder: EnumDecoder) -> Self {
        let EnumDecoder {
            ident,
            to_enum,
            from_enum,
            values,
            type_id,
        } = decoder;

        quote! {
            #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
            pub enum #ident {
                #(#values),*
            }

            impl From<#ident> for String {
                fn from(val: #ident) -> Self {
                    match val {
                        #(#from_enum)*
                        _ => panic!("Unrecognized enum value."),
                    }
                }
            }

            impl From<String> for #ident {
                fn from(val: String) -> Self {
                    match val.as_ref() {
                        #(#to_enum)*
                        _ => panic!("Unrecognized enum value."),
                    }
                }
            }
        }
    }
}
