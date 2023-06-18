use async_graphql_parser::types::{
    EnumType, FieldDefinition, ObjectType, ServiceDocument, TypeKind,
    TypeSystemDefinition, UnionType,
};
use fuel_indexer_schema::parser::ParsedGraphQLSchema;
use proc_macro2::TokenStream;
use quote::quote;
use thiserror::Error;

pub(crate) type IndexerMacroResult<T> = Result<T, IndexerMacroError>;

/// Error type returned by decoding operations.
#[derive(Error, Debug)]
pub enum IndexerMacroError {
    #[error("GraphQL parser error: {0:?}")]
    ParseError(#[from] async_graphql_parser::Error),
}

/// A wrapper object for tokens used in converting GraphQL types into Rust tokens.
pub struct Decoder {
    tokens: TokenStream,
}

impl Decoder {
    /// Create a decoder from a GraphQL object.
    pub fn from_object(
        o: ObjectType,
        parser: &ParsedGraphQLSchema,
    ) -> IndexerMacroResult<Self> {
        unimplemented!()
    }

    /// Create a decoder from a GraphQL field.
    pub fn from_field_def(
        f: FieldDefinition,
        parser: &ParsedGraphQLSchema,
    ) -> IndexerMacroResult<Self> {
        unimplemented!()
    }

    /// Create a decoder from a GraphQL enum.
    pub fn from_enum(
        e: EnumType,
        parser: &ParsedGraphQLSchema,
    ) -> IndexerMacroResult<Self> {
        unimplemented!()
    }

    /// Create a decoder from a GraphQL union.
    pub fn from_union(
        o: UnionType,
        parser: &ParsedGraphQLSchema,
    ) -> IndexerMacroResult<Self> {
        unimplemented!()
    }
}

impl From<Decoder> for TokenStream {
    fn from(decoder: Decoder) -> Self {
        quote! {}
    }
}
