use std::collections::HashSet;

use crate::constants::*;
use async_graphql_parser::types::{BaseType, FieldDefinition, Type};
use async_graphql_value::Name;
use fuel_abi_types::abi::program::{ProgramABI, TypeDeclaration};
use fuel_indexer_lib::graphql::{
    list_field_type_name, types::IdCol, ParsedGraphQLSchema,
};
use fuels_code_gen::utils::Source;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

/// Provides a TokenStream to be used for unwrapping `Option`s for external types.
///
/// This is done because traits cannot be implemented on external types due to the orphan rule.
pub fn unwrap_or_default_for_external_type(
    field_type_name: &str,
) -> proc_macro2::TokenStream {
    match field_type_name {
        "Tai64Timestamp" => {
            quote! {
                .unwrap_or(Tai64Timestamp::from({
                    <[u8;8]>::try_from(0u64.to_be_bytes()).expect("Failed to create byte slice from u64")
                }))
            }
        }
        "Identity" => {
            quote! {
                .unwrap_or(Identity::Address(Address::zeroed()))
            }
        }
        _ => panic!("Default is not implemented for {field_type_name}"),
    }
}

/// Whether or not a `TypeDeclaration` is tuple type
pub fn is_tuple_type(typ: &TypeDeclaration) -> bool {
    typ.type_field.as_str().starts_with('(')
}

/// Extract tokens from JSON ABI file
pub fn get_json_abi(abi_path: Option<String>) -> Option<ProgramABI> {
    match abi_path {
        Some(abi) => {
            let src = match Source::parse(abi) {
                Ok(src) => src,
                Err(e) => {
                    proc_macro_error::abort_call_site!(
                        "`abi` must be a file path to valid json abi: {:?}.",
                        e
                    )
                }
            };

            let source = match src.get() {
                Ok(s) => s,
                Err(e) => {
                    proc_macro_error::abort_call_site!(
                        "Could not fetch JSON ABI. {:?}",
                        e
                    )
                }
            };

            match serde_json::from_str(&source) {
                Ok(parsed) => Some(parsed),
                Err(e) => {
                    proc_macro_error::abort_call_site!(
                        "Invalid JSON from ABI spec: {:?}.",
                        e
                    )
                }
            }
        }
        None => None,
    }
}

/// Whether this TypeDeclaration should be used in the codgen
pub fn is_ignored_type(typ: &TypeDeclaration) -> bool {
    is_tuple_type(typ)
}

/// Whether the TypeDeclaration should be used to build struct fields and decoders
pub fn is_non_decodable_type(typ: &TypeDeclaration) -> bool {
    if GENERIC_TYPES.contains(typ.type_field.as_str()) {
        return true;
    }

    if is_ignored_type(typ) {
        return true;
    }

    false
}

/// Derive Ident for decoded type
pub fn decoded_ident(ty: &str) -> Ident {
    format_ident! { "{}_decoded", ty.to_ascii_lowercase() }
}

/// Return type field name for complex type
fn derive_type_field(ty: &TypeDeclaration) -> String {
    ty.type_field
        .split(' ')
        .last()
        .expect("Could not parse TypeDeclaration for Rust name.")
        .to_string()
}

/// Derive Ident for given TypeDeclaration
pub fn rust_type_ident(ty: &TypeDeclaration) -> Ident {
    if ty.components.is_some() {
        if is_tuple_type(ty) {
            proc_macro_error::abort_call_site!(
                "Cannot derive rust_type_ident of tuple type."
            );
        }

        let name = derive_type_field(ty);
        decoded_ident(&name)
    } else {
        let name = ty.type_field.replace(['[', ']'], "_");
        decoded_ident(&name)
    }
}

/// Derive Rust type tokens for a given TypeDeclaration
pub fn rust_type_token(ty: &TypeDeclaration) -> proc_macro2::TokenStream {
    if ty.components.is_some() {
        let ty_str = ty
            .type_field
            .split(' ')
            .last()
            .expect("Could not parse TypeDeclaration for Rust type.")
            .to_string();

        let ident = format_ident! { "{}", ty_str };
        quote! { #ident }
    } else {
        match ty.type_field.as_str() {
            "()" => quote! {},
            "generic T" => quote! {},
            "raw untyped ptr" => quote! {},
            "b256" => quote! { B256 },
            "bool" => quote! { bool },
            "u16" => quote! { u16 },
            "u32" => quote! { u32 },
            "u64" => quote! { u64 },
            "u8" => quote! { u8 },
            "BlockData" => quote! { BlockData },
            "Call" => quote! { Call },
            "Identity" => quote! { Identity },
            "Log" => quote! { Log },
            "LogData" => quote! { LogData },
            "MessageOut" => quote! { MessageOut },
            "Return" => quote! { Return },
            "ScriptResult" => quote! { ScriptResult },
            "Transfer" => quote! { Transfer },
            "TransferOut" => quote! { TransferOut },
            "Panic" => quote! { Panic },
            "Revert" => quote! { Revert },
            "Mint" => quote! { Mint },
            "Burn" => quote! { Burn },
            o if o.starts_with("str[") => quote! { String },
            o => {
                proc_macro_error::abort_call_site!(
                    "Unrecognized primitive type: {:?}.",
                    o
                )
            }
        }
    }
}

/// Whether or not the given token is a Fuel primitive
pub fn is_fuel_primitive(ty: &proc_macro2::TokenStream) -> bool {
    let ident_str = ty.to_string();
    FUEL_PRIMITIVES.contains(ident_str.as_str())
}

/// Whether or not the given token is a Rust primitive
pub fn is_rust_primitive(ty: &proc_macro2::TokenStream) -> bool {
    let ident_str = ty.to_string();
    RUST_PRIMITIVES.contains(ident_str.as_str())
}

/// Given a type ID, a type token, and a type Ident, return a decoder snippet
/// as a set of tokens
pub fn decode_snippet(
    ty_id: usize,
    ty: &proc_macro2::TokenStream,
    name: &Ident,
) -> proc_macro2::TokenStream {
    if is_fuel_primitive(ty) {
        quote! {
            #ty_id => {
                let obj: #ty = bincode::deserialize(&data).expect("Bad bincode.");
                self.#name.push(obj);
            }
        }
    } else if is_rust_primitive(ty) {
        quote! {
            #ty_id => {
                Logger::warn("Skipping primitive decoder.");
            }
        }
    } else {
        quote! {
            #ty_id => {
                let decoded = ABIDecoder::decode_single(&#ty::param_type(), &data).expect("Failed decoding.");
                let obj = #ty::from_token(decoded).expect("Failed detokenizing.");
                self.#name.push(obj);
            }
        }
    }
}

/// A wrapper trait for helper functions such as rust_type_ident,
/// rust_type_token, and rust_type_ident
pub trait Codegen {
    fn decoded_ident(&self) -> Ident;
    fn rust_type_token(&self) -> proc_macro2::TokenStream;
    fn rust_type_ident(&self) -> Ident;
    fn rust_type_token_string(&self) -> String;
}

impl Codegen for TypeDeclaration {
    fn decoded_ident(&self) -> Ident {
        decoded_ident(&self.rust_type_token().to_string())
    }

    fn rust_type_token(&self) -> proc_macro2::TokenStream {
        rust_type_token(self)
    }

    fn rust_type_ident(&self) -> Ident {
        rust_type_ident(self)
    }

    fn rust_type_token_string(&self) -> String {
        self.rust_type_token().to_string()
    }
}

/// Generate tokens for retrieving necessary indexer data through FFI.
pub fn const_item(id: &str, value: &str) -> proc_macro2::TokenStream {
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

/// Generate tokens for retrieving necessary indexer data through FFI.
pub fn field_extractor(
    field_name: proc_macro2::Ident,
    processed_type: ProcessedFieldType,
) -> proc_macro2::TokenStream {
    let ProcessedFieldType {
        field_type_ident,
        base_type,
        inner_type_ident,
        nullable,
        inner_nullable,
        ..
    } = processed_type;

    let item_popper = quote! { let item = vec.pop().expect("Missing item in row."); };

    let field_extractor = match base_type {
        FieldBaseType::Named => {
            if nullable {
                quote! {
                    let #field_name = match item {
                        FtColumn::#field_type_ident(t) => t,
                        _ => panic!("Invalid nullable column type: {:?}.", item),
                    };
                }
            } else {
                quote! {
                    let #field_name = match item {
                        FtColumn::#field_type_ident(t) => match t {
                            Some(inner_type) => { inner_type },
                            None => {
                                panic!("Non-nullable type is returning a None value.")
                            }
                        },
                        _ => panic!("Invalid column type: {:?}.", item),
                    };
                }
            }
        }
        FieldBaseType::List => {
            // Nullable list of nullable elements: [Entity]
            if nullable && inner_nullable {
                quote! {
                    let #field_name = match item {
                        FtColumn::#field_type_ident(list) => match list {
                            Some(list) => {
                                let unwrapped_list: Vec<_> = list.into_iter().map(|item| match item {
                                    FtColumn::#inner_type_ident(t) => t,
                                    _ => panic!("Invalid column type: {:?}.", item),
                                }).collect::<Vec<_>>();
                                Some(unwrapped_list)
                            }
                            None => None,
                        },
                        _ => panic!("Invalid column type: {:?}.", item),
                    };
                }
            // Nullable list of non-nullable elements: [Entity!]
            } else if nullable && !inner_nullable {
                quote! {
                    let #field_name = match item {
                        FtColumn::#field_type_ident(nullable_list) => match nullable_list {
                            Some(list) => {
                                let unwrapped_list: Vec<_> = list.into_iter().map(|item| match item {
                                    FtColumn::#inner_type_ident(t) => match t {
                                        Some(inner_type) => inner_type,
                                        None => panic!("Non-nullable inner type of list is returning a None value."),
                                    },
                                    _ => panic!("Invalid column type: {:?}.", item),
                                }).collect::<Vec<_>>();
                                Some(unwrapped_list)
                            }
                            None => None,
                        },
                        _ => panic!("Invalid column type: {:?}.", item),
                    };
                }

            // Non-nullable list of nullable elements: [Entity]!
            } else if !nullable && inner_nullable {
                quote! {
                    let #field_name = match item {
                        FtColumn::#field_type_ident(list) => match list {
                            Some(list) => {
                                let unwrapped_list: Vec<_> = list.into_iter().map(|item| match item {
                                    FtColumn::#inner_type_ident(t) => t, // will return Option<T>
                                    _ => panic!("Invalid column type: {:?}.", item),
                                }).collect::<Vec<_>>();
                                unwrapped_list
                            }
                            None => panic!("Non-nullable type is returning a None value."),
                        }
                        _ => panic!("Invalid column type: {:?}.", item),
                    };
                }
            // Non-nullable list of non-nullable elements: [Entity!]!
            } else {
                quote! {
                    let #field_name = match item {
                        FtColumn::#field_type_ident(list) => match list {
                            Some(list) => {
                                let unwrapped_list: Vec<_> = list.into_iter().map(|item| match item {
                                    FtColumn::#inner_type_ident(t) => t.expect("Inner type should not be null."),
                                    _ => panic!("Invalid column type: {:?}.", item),
                                }).collect::<Vec<_>>();
                                unwrapped_list
                            }
                            None => panic!("Non-nullable type is returning a None value."),
                        }
                        _ => panic!("Invalid column type: {:?}.", item),
                    };
                }
            }
        }
    };

    quote! {
        #item_popper
        #field_extractor
    }
}

/// The result of a call to `helpers::process_typedef_field`.
pub struct ProcessedTypedefField {
    /// The `Ident` for the processed `FieldDefinition`'s name.
    pub field_name_ident: proc_macro2::Ident,

    /// The tokens for the processed `FieldDefinition`'s extractor.
    ///
    /// This is used in `T::from_row` where `T: Entity`.
    pub extractor: proc_macro2::TokenStream,

    /// The result of a call to `helpers::process_type`.
    pub processed_type_result: ProcessedFieldType,
}

/// Process an object's field and return a group of tokens.
pub fn process_typedef_field(
    parsed: &ParsedGraphQLSchema,
    mut field_def: FieldDefinition,
) -> ProcessedTypedefField {
    let field_name = field_def.name.to_string();
    let processed_type_result = process_type(parsed, &field_def);
    let ProcessedFieldType {
        inner_nullable,
        nullable,
        ..
    } = processed_type_result;

    let field_name_ident = format_ident! {"{field_name}"};
    let field_typ_name = &parsed.scalar_type_for(&field_def);

    if parsed.is_list_field_type(&list_field_type_name(&field_def)) {
        field_def.ty.node = Type {
            base: BaseType::List(Box::new(Type {
                base: BaseType::Named(Name::new(field_typ_name)),
                nullable: inner_nullable,
            })),
            nullable,
        };
    } else {
        field_def.ty.node = Type {
            base: BaseType::Named(Name::new(field_typ_name)),
            nullable,
        };
    }

    let extractor =
        field_extractor(field_name_ident.clone(), processed_type_result.clone());

    ProcessedTypedefField {
        field_name_ident,
        extractor,
        processed_type_result,
    }
}

/// Process a named field into its type tokens, and the Ident for those type tokens.
#[derive(Debug, Clone)]
pub struct ProcessedFieldType {
    /// The tokens for the processed `FieldDefinition`'s type.
    pub field_type_tokens: proc_macro2::TokenStream,

    /// The `Ident` for the processed `FieldDefinition`'s type.
    pub field_type_ident: proc_macro2::Ident,

    /// The `Ident` for the processed `FieldDefinition`'s inner type.
    ///
    /// Only used when processing a `FieldDefinition` whose type is a GraphQL list type.
    pub inner_type_ident: Option<proc_macro2::Ident>,

    /// Whether or not the processed `FieldDefinition`'s type is nullable.
    pub nullable: bool,

    /// Whether or not the processed `FieldDefinition`'s inner type is nullable.
    ///
    /// Only used when processing a `FieldDefinition` whose type is a GraphQL list type.
    pub inner_nullable: bool,

    /// The base type of the processed `FieldDefinition`.
    pub base_type: FieldBaseType,
}

/// The base type of a `FieldDefinition`.
#[derive(Debug, Clone)]
pub enum FieldBaseType {
    /// The named (or non-list) type.
    Named,

    /// A list type.
    List,
}

/// Process a named type into its type tokens, and the Ident for those type tokens.
pub fn process_type(
    parsed: &ParsedGraphQLSchema,
    f: &FieldDefinition,
) -> ProcessedFieldType {
    let typ = &f.ty.node;
    match &typ.base {
        BaseType::Named(t) => {
            // A `TypeDefinition` name and a given `FieldDefinition` name can be the same,
            // but when using FKs, the `FieldDefinition` type name will include a `!` token
            // if the field is required.
            let name = t.to_string().replace('!', "");
            if !parsed.has_type(&name) {
                panic!("Type '{name}' is not defined in the schema.");
            }

            let field_type_name = parsed.scalar_type_for(f);
            let field_type_ident = format_ident! {"{field_type_name}"};
            let field_type_tokens = if typ.nullable {
                quote! { Option<#field_type_ident> }
            } else {
                quote! { #field_type_ident }
            };

            ProcessedFieldType {
                field_type_ident,
                field_type_tokens,
                base_type: FieldBaseType::Named,
                nullable: typ.nullable,
                inner_nullable: false,
                inner_type_ident: None,
            }
        }

        BaseType::List(t) => {
            let name = t.to_string().replace('!', "");
            if !parsed.has_type(&name) {
                panic!("List type '{name}' is not defined in the schema.");
            }

            let field_type_name = parsed.scalar_type_for(f);
            let inner_ident = format_ident! {"{field_type_name}"};

            let field_type_tokens = {
                if typ.nullable && t.nullable {
                    quote! { Option<Vec<Option<#inner_ident>>> }
                } else if typ.nullable && !t.nullable {
                    quote! { Option<Vec<#inner_ident>> }
                } else if !typ.nullable && t.nullable {
                    quote! { Vec<Option<#inner_ident>> }
                } else {
                    quote! { Vec<#inner_ident> }
                }
            };

            ProcessedFieldType {
                field_type_ident: format_ident! { "Array" },
                field_type_tokens,
                base_type: FieldBaseType::List,
                nullable: typ.nullable,
                inner_nullable: t.nullable,
                inner_type_ident: Some(inner_ident),
            }
        }
    }
}

/// Get tokens for a field's `.clone()`.
pub fn clone_tokens(
    field_typ_name: &str,
    field_id: &str,
    parsed: &ParsedGraphQLSchema,
) -> TokenStream {
    if COPY_TYPES.contains(field_typ_name) {
        return quote! {.clone()};
    }

    if parsed.is_list_field_type(field_id) {
        return quote! {.clone()};
    }

    quote! {}
}

/// Get tokens for a field's `.unwrap_or_default()`.
pub fn unwrap_or_default_tokens(field_typ_name: &str, nullabel: bool) -> TokenStream {
    if nullabel {
        if EXTERNAL_FIELD_TYPES.contains(field_typ_name) {
            unwrap_or_default_for_external_type(field_typ_name)
        } else {
            quote! { .unwrap_or_default() }
        }
    } else {
        quote! {}
    }
}

/// Get tokens for a given field type's `.to_bytes()`.
pub fn to_bytes_tokens(
    field_typ_name: &str,
    processed_type_result: &ProcessedFieldType,
) -> TokenStream {
    let ProcessedFieldType { base_type, .. } = &processed_type_result;
    match base_type {
        FieldBaseType::Named => {
            if EXTERNAL_FIELD_TYPES.contains(field_typ_name) {
                match field_typ_name {
                    "Identity" => quote! { .0 },
                    "Tai64Timestamp" => quote! { .0.to_le_bytes() },
                    _ => panic!("From<{field_typ_name}> not implemented for AsRef<u8>."),
                }
            } else if !ASREF_BYTE_TYPES.contains(field_typ_name) {
                quote! { .to_le_bytes() }
            } else {
                quote! {}
            }
        }
        FieldBaseType::List => {
            // TODO: https://github.com/FuelLabs/fuel-indexer/issues/1063
            quote! {}
        }
    }
}

/// Get tokens for hasher from which to derive a unique ID for this object.
pub fn hasher_tokens(
    field_type_scalar_name: &str,
    field_name: &str,
    base_type: &FieldBaseType,
    hasher: &TokenStream,
    clone: &TokenStream,
    unwrap_or_default: &TokenStream,
    to_bytes: &TokenStream,
) -> Option<TokenStream> {
    match base_type {
        FieldBaseType::Named => {
            let ident = format_ident! {"{field_name}"};
            if !NON_DIGESTIBLE_FIELD_TYPES.contains(field_type_scalar_name) {
                return Some(
                    quote! { #hasher.chain_update(#ident #clone #unwrap_or_default #to_bytes) },
                );
            }
            None
        }
        FieldBaseType::List => None,
    }
}

/// Get tokens for parameters used in `::new()` function and `::get_or_create()`
/// function/method signatures.
pub fn parameters_tokens(
    parameters: &TokenStream,
    field_name: &Ident,
    typ_tokens: &TokenStream,
) -> TokenStream {
    let ident = format_ident! {"{field_name}"};
    quote! { #parameters #ident: #typ_tokens, }
}

/// Get tokens for a field decoder.
pub fn field_decoder_tokens(
    field_name: &Ident,
    clone: &TokenStream,
    processed_type_result: &ProcessedFieldType,
) -> TokenStream {
    let ProcessedFieldType {
        field_type_ident,
        inner_type_ident,
        base_type,
        nullable,
        inner_nullable,
        ..
    } = &processed_type_result;

    match base_type {
        FieldBaseType::Named => {
            if *nullable {
                quote! { FtColumn::#field_type_ident(self.#field_name #clone), }
            } else {
                quote! { FtColumn::#field_type_ident(Some(self.#field_name #clone)), }
            }
        }
        // `FieldBaseType::List` is pretty much similar to `FieldBaseType::Named`. The main difference is, that
        // we need to convert each inner type `T` into a `FtColumn::T`.
        //
        // This prevents us from having to use `FtColumn` in struct fields.
        FieldBaseType::List => {
            let inner_type_ident =
                inner_type_ident.to_owned().expect("Missing inner type.");
            if *nullable {
                if *inner_nullable {
                    quote! { FtColumn::#field_type_ident(self.#field_name.as_ref().map(|items| items.iter().filter_map(|x| {
                        if x.is_none() {
                            return None;
                        }
                        Some(FtColumn::#inner_type_ident(x.to_owned()))
                    }).collect::<Vec<FtColumn>>())), }
                } else {
                    quote! { FtColumn::#field_type_ident(self.#field_name.as_ref().map(|items| items.iter().map(|x| FtColumn::#inner_type_ident(Some(x.to_owned()))).collect::<Vec<FtColumn>>())), }
                }
            } else if *inner_nullable {
                quote! { FtColumn::#field_type_ident(Some(self.#field_name.iter().filter_map(|x| {
                    if x.is_none() {
                        return None;
                    }
                    Some(FtColumn::#inner_type_ident(x.to_owned()))
                }).collect::<Vec<FtColumn>>())), }
            } else {
                quote! { FtColumn::#field_type_ident(Some(self.#field_name.iter().map(|x| FtColumn::#inner_type_ident(Some(x.to_owned()))).collect::<Vec<FtColumn>>())), }
            }
        }
    }
}

/// Whether a given field is eligible for autogenerated ID, where the ID
/// will be derived from the struct's field's values.
pub fn can_derive_id(field_set: &HashSet<String>, field_name: &str) -> bool {
    field_set.contains(IdCol::to_lowercase_str())
        && field_name != IdCol::to_lowercase_str()
}
