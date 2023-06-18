use crate::helpers::*;
use crate::validator::GraphQLSchemaValidator;
use async_graphql_parser::types::{
    EnumType, FieldDefinition, ObjectType, ServiceDocument, TypeKind,
    TypeSystemDefinition, UnionType,
};
use fuel_indexer_schema::parser::ParsedGraphQLSchema;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::{BTreeMap, HashSet};

/// A wrapper object for tokens used in converting GraphQL types into Rust tokens.
#[derive(Default)]
pub struct Decoder {
    struct_fields: TokenStream,
    field_extractors: TokenStream,
    from_row: TokenStream,
    to_row: TokenStream,
    parameters: TokenStream,
    hasher: TokenStream,
    impl_new_fields: TokenStream,
}

impl Decoder {
    /// Create a decoder from a GraphQL object.
    pub fn from_object(
        obj_name: String,
        o: ObjectType,
        parser: &ParsedGraphQLSchema,
    ) -> Self {
        let mut struct_fields = quote! {};
        let mut field_extractors = quote! {};
        let mut from_row = quote! {};
        let mut to_row = quote! {};
        let mut parameters = quote! {};
        let mut hasher = quote! {};
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
            let (typ_tokens, field_name, scalar_typ_name, extractor) =
                foo_process_field(parser, &field.node, &obj_name, FieldKind::Unknown);
            let fieldkind = field_kind(&scalar_typ_name.to_string(), parser);

            // Process a second time to convert any complex/special field types into regular scalars
            let (typ_tokens, field_name, scalar_typ_name, extractor) =
                foo_process_field(parser, &field.node, &obj_name, fieldkind);

            fields_map.insert(field_name.to_string(), scalar_typ_name.to_string());

            let clone = clone_tokens(&scalar_typ_name.to_string());
            let field_decoder = field_type_decoder(
                field.node.ty.node.nullable,
                &scalar_typ_name.to_string(),
                &field_name,
                clone,
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
        }
        Self::default()
    }

    /// Create a decoder from a GraphQL enum.
    pub fn from_enum(
        enum_name: String,
        e: EnumType,
        parser: &ParsedGraphQLSchema,
    ) -> Self {
        Self::default()
    }

    /// Create a decoder from a GraphQL union.
    pub fn from_union(
        union_name: String,
        u: UnionType,
        parser: &ParsedGraphQLSchema,
    ) -> Self {
        Self::default()
    }
}

impl From<Decoder> for TokenStream {
    fn from(decoder: Decoder) -> Self {
        quote! {}
    }
}
