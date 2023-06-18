use crate::validator::GraphQLSchemaValidator;
use crate::{constants::*, helpers::*};
use async_graphql_parser::types::{
    EnumType, FieldDefinition, ObjectType, ServiceDocument, TypeKind,
    TypeSystemDefinition, UnionType,
};
use fuel_indexer_database_types::IdCol;
use fuel_indexer_schema::parser::ParsedGraphQLSchema;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, HashSet};
use syn::Ident;

pub enum ExecutionSource {
    Native,
    Wasm,
}

/// A wrapper object for tokens used in converting GraphQL types into Rust tokens.
pub struct Decoder {
    ident: Ident,
    struct_fields: TokenStream,
    field_extractors: TokenStream,
    from_row: TokenStream,
    to_row: TokenStream,
    parameters: TokenStream,
    hasher: TokenStream,
    impl_new_fields: TokenStream,
    impl_new_params: ImplNewParameters,
    source: ExecutionSource,
    type_id: i64,
}

impl Default for Decoder {
    fn default() -> Self {
        Self {
            ident: format_ident!("Decoder"),
            struct_fields: quote! {},
            field_extractors: quote! {},
            from_row: quote! {},
            to_row: quote! {},
            parameters: quote! {},
            hasher: quote! {},
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

impl Decoder {
    /// Create a decoder from a GraphQL object.
    pub fn from_object(
        obj_name: String,
        o: ObjectType,
        parser: &ParsedGraphQLSchema,
    ) -> Self {
        let ident = format_ident!("{}", obj_name);
        let type_id = typekind_type_id(&parser.namespace, &parser.identifier, &obj_name);

        let mut struct_fields = quote! {};
        let mut field_extractors = quote! {};
        let mut from_row = quote! {};
        let mut to_row = quote! {};
        let mut parameters = quote! {};
        let mut hasher = quote! { Sha256::new() };
        let mut impl_new_fields = quote! {};

        let mut fields_map = BTreeMap::new();
        let obj_fields = parser
            .object_field_mappings
            .get(&obj_name)
            .expect("")
            .iter()
            .map(|(k, v)| k.to_owned())
            .collect::<HashSet<String>>();
        GraphQLSchemaValidator::check_disallowed_typekind_name(&obj_name);

        for field in &o.fields {
            // Process once to get the general field info
            let (typ_tokens, field_name, field_typ_name, extractor) = foo_process_field(
                parser,
                field.node.clone(),
                &obj_name,
                FieldKind::Unknown,
            );

            let fieldkind = field_kind(&field_typ_name.to_string(), parser);

            // Process a second time to convert any complex/special field types into regular scalars
            let (typ_tokens, field_name, field_type_scalar_name, extractor) =
                foo_process_field(parser, field.node.clone(), &obj_name, fieldkind);

            fields_map.insert(field_name.to_string(), field_type_scalar_name.to_string());

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
                parameters = parameters_tokens(parameters, &field_name, typ_tokens);
                hasher = hasher_tokens(
                    &field_type_scalar_name.to_string(),
                    hasher,
                    &field_name,
                    clone,
                    unwrap_or_default,
                    to_bytes,
                );

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
                is_native: parser.is_native,
                field_set: obj_fields,
            },
            type_id,
        }
    }

    /// Create a decoder from a GraphQL enum.
    pub fn from_enum(
        enum_name: String,
        e: EnumType,
        parser: &ParsedGraphQLSchema,
    ) -> Self {
        unimplemented!()
    }

    /// Create a decoder from a GraphQL union.
    pub fn from_union(
        union_name: String,
        u: UnionType,
        parser: &ParsedGraphQLSchema,
    ) -> Self {
        unimplemented!()
    }
}

impl From<Decoder> for TokenStream {
    fn from(decoder: Decoder) -> Self {
        let Decoder {
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
