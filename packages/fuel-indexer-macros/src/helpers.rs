use std::collections::{HashMap, HashSet};

use async_graphql_parser::types::{BaseType, FieldDefinition, Type as AsyncGraphQLType};
use async_graphql_value::Name;
use fuel_abi_types::abi::program::{
    ABIFunction, LoggedType, ProgramABI, TypeDeclaration,
};
use fuel_indexer_lib::{
    constants::*,
    graphql::{list_field_type_name, types::IdCol, ParsedGraphQLSchema},
};
use fuel_indexer_types::{type_id, FUEL_TYPES_NAMESPACE};
use fuels_code_gen::utils::Source;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{GenericArgument, Ident, PathArguments, Type, TypePath};

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

/// Whether a `TypeDeclaration` is tuple type
pub fn is_tuple_type(typ: &TypeDeclaration) -> bool {
    let mut type_field_chars = typ.type_field.chars();
    type_field_chars.next().is_some_and(|c| c == '(')
        && type_field_chars.next().is_some_and(|c| c != ')')
}

/// Whether a `TypeDeclaration` is a unit type
pub fn is_unit_type(typ: &TypeDeclaration) -> bool {
    let mut type_field_chars = typ.type_field.chars();
    type_field_chars.next().is_some_and(|c| c == '(')
        && type_field_chars.next().is_some_and(|c| c == ')')
}

/// Whether the TypeDeclaration should be used to build struct fields and decoders
pub fn is_non_decodable_type(typ: &TypeDeclaration) -> bool {
    is_tuple_type(typ)
        || is_unit_type(typ)
        || IGNORED_GENERIC_METADATA.contains(typ.type_field.as_str())
}

/// Derive Ident for decoded type
///
/// These idents are used as fields for the `Decoder` struct.
fn decoded_ident(typ: &TypeDeclaration) -> Ident {
    let name = {
        let name = derive_type_name(typ);
        if typ.components.is_some() {
            if is_array_type(typ) {
                let name = name.replace(['[', ']', ' '], "");
                let name = name.replace(';', "_");
                format!("array_{}", name)
            } else {
                name
            }
        } else if name.starts_with("Option") {
            typ.type_field.replace(['<', '>'], "_")
        } else {
            typ.type_field.replace(['[', ']'], "_")
        }
    };

    if is_generic_type(typ) {
        let gt = GenericType::from(name.as_str());
        match gt {
            GenericType::Vec => {
                let name = name.replace(['<', '>'], "_").to_ascii_lowercase();
                format_ident! { "{}decoded", name }
            }
            GenericType::Option => {
                let name = name.replace(['<', '>'], "_").to_ascii_lowercase();
                format_ident! { "{}decoded", name }
            }
            _ => proc_macro_error::abort_call_site!(
                "Could not derive decoded ident for generic type: {:?}.",
                name
            ),
        }
    } else {
        format_ident! { "{}_decoded", name.to_ascii_lowercase() }
    }
}

/// Given a `TypeDeclaration`, return name of the base of its typed path.
///
/// `Vec<T>` returns `Vec`, `Option<T>` returns `Option`, `u8` returns `u8`, etc.
pub fn derive_type_name(typ: &TypeDeclaration) -> String {
    if is_array_type(typ) {
        typ.type_field.clone()
    } else {
        typ.type_field
            .split(' ')
            .last()
            .expect("Type field name expected")
            .to_string()
    }
}

/// Whether or not the given token is a Fuel primitive
///
/// These differ from `RESERVED_TYPEDEF_NAMES` in that `FUEL_PRIMITIVES` are type names
/// that are checked against the contract JSON ABI, while `RESERVED_TYPEDEF_NAMES` are
/// checked against the GraphQL schema.
pub fn is_fuel_primitive(typ: &TypeDeclaration) -> bool {
    let name = derive_type_name(typ);
    FUEL_PRIMITIVES.contains(name.as_str())
}

/// Whether or not the given token is a Rust primitive
fn is_rust_primitive(ty: &proc_macro2::TokenStream) -> bool {
    let ident_str = ty.to_string();
    match ident_str.as_str() {
        "u8" | "u16" | "u32" | "u64" | "bool" | "String" => true,
        _ => {
            ident_str.starts_with('[')
                && ident_str.ends_with(']')
                && ident_str.contains(';')
        }
    }
}

/// Whether or not the given tokens are a generic type
pub fn is_generic_type(typ: &TypeDeclaration) -> bool {
    let gt = GenericType::from(typ);
    matches!(gt, GenericType::Vec | GenericType::Option)
}

/// Given a `TokenStream` representing this `TypeDeclaration`'s fully typed path,
/// return the associated `match` arm for decoding this type in the `Decoder`.
pub fn decode_snippet(
    type_tokens: &proc_macro2::TokenStream,
    typ: &TypeDeclaration,
) -> proc_macro2::TokenStream {
    let name = typ.decoder_field_ident();
    let ty_id = typ.type_id;

    if is_fuel_primitive(typ) {
        quote! {
            #ty_id => {
                let obj: #type_tokens = bincode::deserialize(&data).expect("Bad bincode.");
                self.#name.push(obj);
            }
        }
    } else if is_rust_primitive(type_tokens) {
        quote! {
            #ty_id => {
                Logger::warn("Skipping primitive decoder.");
            }
        }
    } else if is_generic_type(typ) {
        let gt = GenericType::from(typ);
        match gt {
            GenericType::Vec => {
                // https://github.com/FuelLabs/fuel-indexer/issues/503
                quote! {
                    #ty_id => {
                        Logger::warn("Skipping unsupported vec decoder.");
                    }
                }
            }
            GenericType::Option => {
                let (inner, typ) = inner_typedef(typ);
                let inner = format_ident! { "{}", inner };
                quote! {
                    #ty_id => {
                        let decoded = ABIDecoder::decode_single(&#typ::<#inner>::param_type(), &data).expect("Failed decoding.");
                        let obj = #typ::<#inner>::from_token(decoded).expect("Failed detokenizing.");
                        self.#name.push(obj);
                    }
                }
            }
            _ => proc_macro_error::abort_call_site!(
                "Decoder snippet is unsupported for generic type: {:?}.",
                gt
            ),
        }
    } else {
        quote! {
            #ty_id => {
                let decoded = ABIDecoder::decode_single(&#type_tokens::param_type(), &data).expect("Failed decoding.");
                let obj = #type_tokens::from_token(decoded).expect("Failed detokenizing.");
                self.#name.push(obj);
            }
        }
    }
}

/// A hacky wrapper trait for helper functions.
pub trait Codegen {
    /// Return the derived name for this `TypeDeclaration`.
    fn name(&self) -> String;

    /// Return the `TokenStream` for this `TypeDeclaration`.
    fn rust_tokens(&self) -> proc_macro2::TokenStream;

    /// Return the `Ident` for this `TypeDeclaration`'s decoder field.
    fn decoder_field_ident(&self) -> Ident;
}

impl Codegen for TypeDeclaration {
    fn name(&self) -> String {
        derive_type_name(self)
    }

    fn rust_tokens(&self) -> proc_macro2::TokenStream {
        // Array works a bit differently where it's not a complex type (it's a rust primitive),
        // but we still have to format each part of the array (type and size) separately.
        if is_array_type(self) {
            let name = derive_type_name(self).replace(['[', ']', ';'], "");
            let mut iter = name.split(' ');
            let ty = iter.next().unwrap();
            let size = iter.next().unwrap().parse::<usize>().unwrap();

            let ty = format_ident! { "{}", ty };

            quote! { [#ty; #size] }
        } else if self.components.is_some() {
            let name = derive_type_name(self);
            let ident = format_ident! { "{}", name };
            quote! { #ident }
        } else {
            match self.type_field.as_str() {
                "()" => quote! {},
                "b256" => quote! { B256 },
                "BlockData" => quote! { BlockData },
                "bool" => quote! { bool },
                "Burn" => quote! { Burn },
                "Call" => quote! { Call },
                "generic T" => quote! {},
                "Identity" => quote! { Identity },
                "Log" => quote! { Log },
                "LogData" => quote! { LogData },
                "MessageOut" => quote! { MessageOut },
                "Mint" => quote! { Mint },
                "Panic" => quote! { Panic },
                "Return" => quote! { Return },
                "Revert" => quote! { Revert },
                "ScriptResult" => quote! { ScriptResult },
                "Transfer" => quote! { Transfer },
                "TransferOut" => quote! { TransferOut },
                "u16" => quote! { u16 },
                "u32" => quote! { u32 },
                "u64" => quote! { u64 },
                "u8" => quote! { u8 },
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

    fn decoder_field_ident(&self) -> Ident {
        decoded_ident(self)
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
        field_def.ty.node = AsyncGraphQLType {
            base: BaseType::List(Box::new(AsyncGraphQLType {
                base: BaseType::Named(Name::new(field_typ_name)),
                nullable: inner_nullable,
            })),
            nullable,
        };
    } else {
        field_def.ty.node = AsyncGraphQLType {
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

            if NON_DIGESTIBLE_FIELD_TYPES.contains(field_type_scalar_name) {
                return None;
            }

            Some(
                quote! { #hasher.chain_update(#ident #clone #unwrap_or_default #to_bytes) },
            )
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

/// Strip the call path from the type field of a `TypeDeclaration`.
///
/// It is possible that the type field for a `TypeDeclaration` contains a
/// fully-qualified path (e.g. `std::address::Address` as opposed to `Address`).
/// Path separators are not allowed to be used as part of an identifier, so this
/// function removes the qualifying path while keeping the type keyword.
pub fn strip_callpath_from_type_field(mut typ: TypeDeclaration) -> TypeDeclaration {
    if is_non_decodable_type(&typ) {
        return typ;
    }

    let mut s = typ.type_field.split_whitespace();
    typ.type_field =
        if let (Some(keyword), Some(fully_qualified_type_path)) = (s.next(), s.last()) {
            if let Some(slug) = fully_qualified_type_path.split("::").last() {
                [keyword, slug].join(" ")
            } else {
                unreachable!("All types should be formed with a keyword and call path")
            }
        } else {
            typ.type_field
        };
    typ
}

/// Simply represents a value for a generic type.
#[derive(Debug)]
pub enum GenericType {
    /// `Vec<T>`
    Vec,

    /// `Option<T>`
    Option,
    #[allow(unused)]
    Other,
}

impl From<&TypeDeclaration> for GenericType {
    fn from(t: &TypeDeclaration) -> Self {
        Self::from(derive_type_name(t))
    }
}

impl From<String> for GenericType {
    fn from(s: String) -> Self {
        if s.starts_with("Vec") {
            GenericType::Vec
        } else if s.starts_with("Option") {
            GenericType::Option
        } else {
            GenericType::Other
        }
    }
}

impl From<GenericType> for &str {
    fn from(t: GenericType) -> Self {
        match t {
            GenericType::Vec => "Vec",
            GenericType::Option => "Option",
            GenericType::Other => unimplemented!("Generic type not implemented."),
        }
    }
}

impl From<&str> for GenericType {
    fn from(s: &str) -> Self {
        if s.starts_with("Vec") {
            GenericType::Vec
        } else if s.starts_with("Option") {
            GenericType::Option
        } else {
            GenericType::Other
        }
    }
}

impl From<GenericType> for TokenStream {
    fn from(t: GenericType) -> Self {
        match t {
            GenericType::Vec => quote! { Vec },
            GenericType::Option => quote! { Option },
            GenericType::Other => unimplemented!("Generic type not implemented."),
        }
    }
}

/// Same as `derive_generic_inner_typedefs` but specifically for log types.
///
/// Where as `derive_generic_inner_typedefs` can return multiple inner types, this
/// only returns the single inner type associated with this log specific logged type
pub fn derive_log_generic_inner_typedefs<'a>(
    typ: &'a LoggedType,
    abi: &ProgramABI,
    abi_types: &'a HashMap<usize, TypeDeclaration>,
) -> &'a TypeDeclaration {
    let result =
        abi.logged_types
            .iter()
            .flatten()
            .filter_map(|log| {
                if log.log_id == typ.log_id && log.application.type_arguments.is_some() {
                    let args = log.application.type_arguments.as_ref().unwrap();
                    let inner = args.first().expect("No type args found.");
                    return Some(abi_types.get(&inner.type_id).unwrap_or_else(|| {
                        panic!("Inner type not in ABI: {:?}", inner)
                    }));
                }
                None
            })
            .collect::<Vec<_>>();

    result.first().expect("No inner type found.")
}

/// Derive the inner ident names for collections types.
///
/// Given a `GenericType`, and ABI JSON metadata, derive the inner `TypeDefinition`s associated with the given
/// generic `TypeDefinition`.
///
/// So this function will parse all function inputs/outputs and log types, find all generics (e.g., `Vec<T>`, `Option<U>`),
/// and return the inner `TypeDefinition`s associated with those generics (e.g., `T` and `U`)
pub fn derive_generic_inner_typedefs<'a>(
    typ: &'a TypeDeclaration,
    funcs: &[ABIFunction],
    log_types: &[LoggedType],
    abi_types: &'a HashMap<usize, TypeDeclaration>,
) -> Vec<&'a TypeDeclaration> {
    let name = typ.type_field.split(' ').last().unwrap();
    let t = GenericType::from(name);

    // Per Ahmed from fuels-rs:
    //
    // "So if you wish to see all the various Ts used with SomeStruct<T> (in this case Vec<T>)
    // you have no choice but to go through all functions and find inputs/outputs that reference
    // SomeStruct<T> and see what the typeArguments are. Those will replace the typeParameters inside
    // the type declaration."
    match t {
        GenericType::Option | GenericType::Vec => {
            let mut typs = funcs
                .iter()
                .flat_map(|func| {
                    func.inputs
                        .iter()
                        .filter_map(|i| {
                            if i.type_id == typ.type_id && i.type_arguments.is_some() {
                                let args = i.type_arguments.as_ref().unwrap();
                                let inner = args.first().expect("No type args found.");
                                return Some(
                                    abi_types.get(&inner.type_id).unwrap_or_else(|| {
                                        panic!("Inner type not in ABI: {:?}", inner)
                                    }),
                                );
                            }
                            None
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            let mut output_typs = funcs
                .iter()
                .flat_map(|func| {
                    if func.output.type_id == typ.type_id
                        && func.output.type_arguments.is_some()
                    {
                        let args = func.output.type_arguments.as_ref().unwrap();
                        let inner = args.first().expect("No type args found.");
                        return Some(abi_types.get(&inner.type_id).unwrap_or_else(
                            || panic!("Inner type not in ABI: {:?}", inner),
                        ));
                    }
                    None
                })
                .collect::<Vec<_>>();

            // Parse these as well because we will need to add them to our
            // mapping of type IDs and `TypeDeclaration`s so we can use them when
            // we parse log types.
            let mut log_types = log_types
                .iter()
                .filter_map(|log| {
                    if log.application.type_id == typ.type_id
                        && log.application.type_arguments.is_some()
                    {
                        let args = log.application.type_arguments.as_ref().unwrap();
                        let inner = args.first().expect("No type args found.");
                        return Some(abi_types.get(&inner.type_id).unwrap_or_else(
                            || panic!("Inner type not in ABI: {:?}", inner),
                        ));
                    }
                    None
                })
                .collect::<Vec<_>>();

            typs.append(&mut output_typs);
            typs.append(&mut log_types);
            typs
        }
        _ => proc_macro_error::abort_call_site!(
            "Can't derive idents for unsupported generic type: {:?}",
            t
        ),
    }
}

/// Extract the full type ident from a given path.
pub fn typed_path_name(p: &TypePath) -> String {
    let base = p
        .path
        .segments
        .last()
        .expect("Could not get last path segment.");

    let base_name = base.ident.to_string();

    if GENERIC_STRUCTS.contains(base_name.as_str()) {
        typed_path_string(p)
    } else {
        base_name
    }
}

/// Extract the fully typed path for this generic type.
///
/// When given a generic's `TypedPath` (e.g., `Vec<T>`), we need to extract the struct type (e.g., `Vec`)
/// and also inner type `T`.
///
/// The following assumes a generic type definition format of:
///      `$struct_name $bracket(open) $inner_t_name  $bracket(close)` (e.g., `Vec<T>` or `Option<T>`)
fn typed_path_string(p: &TypePath) -> String {
    let mut result = String::new();

    let base = p
        .path
        .segments
        .last()
        .expect("Could not get last path segment.");

    result.push_str(&base.ident.to_string());
    result.push('<');

    match base.arguments {
        PathArguments::AngleBracketed(ref inner) => {
            let _ = inner
                .args
                .iter()
                .map(|arg| match arg {
                    GenericArgument::Type(Type::Path(p)) => {
                        let segment = p
                            .path
                            .segments
                            .last()
                            .expect("Could not get last path segment.");
                        let name = segment.ident.to_string();
                        result.push_str(&name);
                        result.push('>');
                    }
                    _ => panic!("Unsupported generic argument."),
                })
                .collect::<Vec<_>>();
        }
        _ => panic!("Unsupported generic argument."),
    }
    result
}

/// Retrieve the inner `T` from the given generic `TypeDefinition`s type field.
///
/// E.g., extrac `T` from `Vec<T>`
pub fn inner_typedef(typ: &TypeDeclaration) -> (String, proc_macro2::TokenStream) {
    let name = derive_type_name(typ);
    let gt = GenericType::from(name.clone());
    match gt {
        GenericType::Vec => (name[4..name.len() - 1].to_string(), gt.into()),
        GenericType::Option => (name[7..name.len() - 1].to_string(), gt.into()),
        _ => proc_macro_error::abort_call_site!("Unsupported generic type: {:?}", gt),
    }
}

/// Derive the output type ID for a given ABI function.
///
/// For non-generic `TypeDeclaration`s, we can just return the `type_id` associated with
/// that function output. But for generic `TypeDeclaration`s, we need to derive the `type_id`
/// using the fully typed path of the generic type (e.g., `Vec<T>`) in order to match
/// the type ID of the `TypeDefinition` we manually inserted into the `#[indexer]` `abi_types_tyid`
/// mapping.
pub fn function_output_type_id(
    f: &ABIFunction,
    abi_types: &HashMap<usize, TypeDeclaration>,
) -> usize {
    let outer_typ = abi_types.get(&f.output.type_id).unwrap_or_else(|| {
        panic!(
            "function_output_type_id: Type with TypeID({}) is missing from the JSON ABI",
            f.output.type_id
        )
    });
    if is_generic_type(outer_typ) {
        let name = derive_type_name(outer_typ);
        let gt = GenericType::from(name);
        let inner = f
            .output
            .type_arguments
            .as_ref()
            .unwrap()
            .first()
            .expect("Missing inner type.");

        match gt {
            GenericType::Option | GenericType::Vec => {
                let inner_typ = abi_types.get(&inner.type_id).unwrap_or_else(|| {
                    panic!("function_output_type_id: Generic inner type with TypeID({}) is missing from the JSON ABI", inner.type_id)
                });
                let (typ_name, _) =
                    typed_path_components(outer_typ, inner_typ, abi_types);
                type_id(FUEL_TYPES_NAMESPACE, &typ_name) as usize
            }
            _ => proc_macro_error::abort_call_site!(
                "Unsupported generic type for function outputs: {:?}",
                gt
            ),
        }
    } else {
        f.output.type_id
    }
}

/// Given two `TypeDeclaration`s for a generic struct (e.g., `Vec`) and an inner type (e.g., `T`),
/// return the associated fully typed path. (e.g., `Vec<T>`)
///
/// We do this by recursively deriving the inner `T` for the provided inner `T`, so long as
/// the inner `T` is itself, generic.
pub fn typed_path_components(
    outer: &TypeDeclaration,
    inner: &TypeDeclaration,
    abi_types: &HashMap<usize, TypeDeclaration>,
) -> (String, TokenStream) {
    let outer_name = derive_type_name(outer);
    let outer_ident = format_ident! { "{}", outer_name };
    let inner_name = derive_type_name(inner);
    let inner_ident = format_ident! { "{}", inner_name };
    let mut tokens = quote! { #inner_ident };

    let mut curr = inner;
    while is_generic_type(curr) {
        let gt = GenericType::from(curr);
        match gt {
            GenericType::Option | GenericType::Vec => {
                curr = abi_types.get(&inner.type_id).unwrap_or_else(|| {
                    panic!("typed_path_components: Generic inner type with TypeID({}) is missing from the JSON ABI", inner.type_id)
                });
                let name = derive_type_name(curr);
                let ident = format_ident! { "{}", name };
                tokens = quote! { #tokens<#ident> }
            }
            _ => proc_macro_error::abort_call_site!(
                "Unsupported generic type for typed path: {:?}",
                gt
            ),
        }
    }

    tokens = quote! { #outer_ident<#tokens> };

    // Remove white space from path`
    let name = tokens.to_string().replace(' ', "");

    (name, tokens)
}

pub fn is_array_type(typ: &TypeDeclaration) -> bool {
    typ.type_field.starts_with('[')
        && typ.type_field.ends_with(']')
        && typ.type_field.contains(';')
}

/// Determine whether or not the given type name is an unsupported type.
///
/// Since we allow unsupported types in the ABI JSON, this check is only
/// performed on indexer handler function arg typed paths.
pub fn is_unsupported_type(type_name: &str) -> bool {
    let gt = GenericType::from(type_name);
    match gt {
        GenericType::Vec => UNSUPPORTED_ABI_JSON_TYPES.contains(gt.into()),
        _ => UNSUPPORTED_ABI_JSON_TYPES.contains(type_name),
    }
}
