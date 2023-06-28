use std::collections::HashSet;

use crate::constants::*;
use async_graphql_parser::types::{BaseType, FieldDefinition, Type, TypeDefinition};
use async_graphql_value::Name;
use fuel_abi_types::abi::program::{ProgramABI, TypeDeclaration};
use fuel_indexer_lib::graphql::{
    extract_foreign_key_info, field_id, types::IdCol, ParsedGraphQLSchema,
};
use fuels_code_gen::utils::Source;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

/// Provides a TokenStream to be used as a conversion to bytes
/// for external types; this is done because traits cannot be
/// implemented on external types due to the orphan rule.
pub fn to_bytes_method_for_external_type(
    field_type_name: &str,
) -> proc_macro2::TokenStream {
    match field_type_name {
        "Identity" => quote! { .0 },
        "Tai64Timestamp" => quote! { .0.to_le_bytes() },
        _ => panic!("From<{field_type_name}> not implemented for AsRef<u8>."),
    }
}

/// Provides a TokenStream to be used for unwrapping `Option`s
/// for external types; this is done because traits cannot be
/// implemented on external types due to the orphan rule.
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

/// If TypeDeclaration is tuple type
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
    if is_tuple_type(typ) {
        return true;
    }
    false
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
    schema: &ParsedGraphQLSchema,
    field_name: proc_macro2::Ident,
    mut field_type: proc_macro2::Ident,
    processed_type: ProcessTypeResult,
) -> proc_macro2::TokenStream {
    let type_name = field_type.to_string();
    let item_popper = quote! { let item = vec.pop().expect("Missing item in row."); };

    let ProcessTypeResult {
        field_type_ident,
        field_type_tokens,
        base_type,
        inner_type_ident,
        nullable,
        inner_nullable,
    } = processed_type;

    let field_extractor = match base_type {
        FieldBaseType::Named => {
            if nullable {
                quote! {
                    let #field_name = match item {
                        FtColumn::#field_type(t) => t,
                        _ => panic!("Invalid nullable column type: {:?}.", item),
                    };
                }
            } else {
                quote! {
                    let #field_name = match item {
                        FtColumn::#field_type(t) => match t {
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
            let list_type = format_ident! {"List"};
            if nullable && inner_nullable {
                quote! {
                    let #field_name = match item {
                        FtColumn::#list_type(nullable_list) => match nullable_list {
                            Some(list) => {
                                let unwrapped_list: Vec<_> = list.into_iter().map(|item| match item {
                                    FtColumn::#field_type(t) => t,
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
                        FtColumn::#list_type(nullable_list) => match nullable_list {
                            Some(list) => {
                                let unwrapped_list: Vec<_> = list.into_iter().map(|item| match item {
                                    FtColumn::#field_type(t) => match t {
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
                        FtColumn::#list_type(list) => match list {
                            Some(list) => {
                                let unwrapped_list: Vec<_> = list.into_iter().map(|item| match item {
                                    FtColumn::#field_type(t) => t,
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
                        FtColumn::#list_type(list) => match list {
                            Some(list) => {
                                let unwrapped_list: Vec<_> = list.into_iter().map(|item| match item {
                                    FtColumn::#field_type(t) => match t {
                                        Some(inner_type) => inner_type,
                                        None => panic!("Non-nullable inner type of list is returning a None value."),
                                    },
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

/// Type of special fields in GraphQL schema.
#[derive(Debug, Clone)]
pub enum FieldKind {
    /// `ForeignKey` kinds reference other `TypeDefinition`s in the GraphQL schema.
    ForeignKey,

    /// `Enum` kinds are GraphQL enums (converted into `Charfield` or String types).
    Enum,

    /// `Virtual` kinds are GraphQL `TypeDefinition`s from which no SQL table is generated.
    Virtual,

    /// `Union` kinds are GraphQL `TypeDefinition`s from which new struct/entities
    /// are derived using the set of each union member's fields.
    Union,

    /// `Scalar` kinds are just scalar types.
    Scalar,

    /// `List` kinds are lists are GraphQL list types who's items are either a `FieldKind::Scalar`
    /// type or a `FieldKind::ForeignKey` type.
    List(Box<FieldKind>),
}

/// Process an object's field and return a group of tokens.
pub fn process_typedef_field(
    parsed: &ParsedGraphQLSchema,
    mut field_def: FieldDefinition,
    typdef: &TypeDefinition,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::Ident,
    proc_macro2::Ident,
    proc_macro2::TokenStream,
) {
    let field_name = field_def.name.to_string();
    let processed_type_result = process_type(parsed, &field_def.ty.node);
    let ProcessTypeResult {
        field_type_ident,
        field_type_tokens,
        base_type,
        inner_type_ident,
        nullable,
        inner_nullable,
    } = processed_type_result.clone();

    let fid = field_id(&typdef.name.to_string(), &field_name);
    let lookup_type = inner_type_ident.unwrap_or(field_type_ident.clone());

    let fieldkind = field_kind(&lookup_type.to_string(), &fid, parsed);
    match fieldkind {
        FieldKind::ForeignKey => {
            let (ref_coltype, _ref_colname, _ref_tablename) =
                extract_foreign_key_info(&field_def, parsed.field_type_mappings());

            // We're manually updated the field type here because we need to substitute the field name
            // into a scalar type name.
            field_def.ty.node = Type {
                base: BaseType::Named(Name::new(ref_coltype)),
                nullable: field_def.ty.node.nullable,
            };

            process_typedef_field(parsed, field_def, typdef)
        }
        FieldKind::Enum => {
            field_def.ty.node = Type {
                base: BaseType::Named(Name::new("Charfield")),
                nullable: field_def.ty.node.nullable,
            };
            process_typedef_field(parsed, field_def, typdef)
        }
        FieldKind::Virtual => {
            field_def.ty.node = Type {
                base: BaseType::Named(Name::new("Virtual")),
                nullable: field_def.ty.node.nullable,
            };
            process_typedef_field(parsed, field_def, typdef)
        }
        FieldKind::Union => {
            let field_typ_name = field_def.ty.to_string().replace(['[', ']', '!'], "");
            match parsed.is_virtual_typedef(&field_typ_name) {
                true => {
                    field_def.ty.node = Type {
                        base: BaseType::Named(Name::new("Virtual")),
                        nullable: field_def.ty.node.nullable,
                    };
                    process_typedef_field(parsed, field_def, typdef)
                }
                false => match parsed.is_possible_foreign_key(&field_typ_name) {
                    true => {
                        // Determine implicit vs explicit FK
                        let (ref_coltype, _ref_colname, _ref_tablename) =
                            extract_foreign_key_info(
                                &field_def,
                                parsed.field_type_mappings(),
                            );

                        field_def.ty.node = Type {
                            base: BaseType::Named(Name::new(ref_coltype)),
                            nullable: field_def.ty.node.nullable,
                        };

                        process_typedef_field(parsed, field_def, typdef)
                    }
                    false => process_typedef_field(parsed, field_def, typdef),
                },
            }
        }
        FieldKind::List(kind) => match *kind {
            FieldKind::ForeignKey => {
                let (ref_coltype, _ref_colname, _ref_tablename) =
                    extract_foreign_key_info(&field_def, parsed.field_type_mappings());

                let inner_nullable = field_def.ty.node.to_string().contains('!')
                    && !field_def.ty.node.to_string().ends_with('!');

                field_def.ty.node = Type {
                    base: BaseType::List(Box::new(Type {
                        base: BaseType::Named(Name::new(&ref_coltype)),
                        nullable: inner_nullable,
                    })),
                    nullable: field_def.ty.node.nullable,
                };

                process_typedef_field(parsed, field_def, typdef)
            }
            FieldKind::Scalar => {
                let field_name_ident = format_ident! {"{field_name}"};
                let extractor = field_extractor(
                    parsed,
                    field_name_ident.clone(),
                    field_type_ident.clone(),
                    processed_type_result.clone(),
                );

                (
                    field_type_tokens,
                    field_name_ident,
                    field_type_ident,
                    extractor,
                )
            }
            _ => unimplemented!("cant reach this"),
        },
        _ => {
            let field_name_ident = format_ident! {"{field_name}"};
            let extractor = field_extractor(
                parsed,
                field_name_ident.clone(),
                field_type_ident.clone(),
                processed_type_result.clone(),
            );

            (
                field_type_tokens,
                field_name_ident,
                field_type_ident,
                extractor,
            )
        }
    }
}

/// Process a named field into its type tokens, and the Ident for those type tokens.
#[derive(Debug, Clone)]
pub struct ProcessTypeResult {
    pub field_type_tokens: proc_macro2::TokenStream,
    pub field_type_ident: proc_macro2::Ident,
    pub inner_type_ident: Option<proc_macro2::Ident>,
    pub nullable: bool,
    pub inner_nullable: bool,
    pub base_type: FieldBaseType,
}

#[derive(Debug, Clone)]
pub enum FieldBaseType {
    Named,
    List,
}

/// Process a named type into its type tokens, and the Ident for those type tokens.
pub fn process_type(parsed: &ParsedGraphQLSchema, typ: &Type) -> ProcessTypeResult {
    match &typ.base {
        BaseType::Named(t) => {
            // A `TypeDefinition` name and a given `FieldDefinition` name can be the same,
            // but when using FKs, the `FieldDefinition` type name will include a `!` token
            // if the field is required.
            let name = t.to_string().replace('!', "");
            if !parsed.has_type(&name) {
                panic!("Type '{name}' is not defined in the schema.");
            }

            let field_type_ident = format_ident! {"{name}"};
            let field_type_tokens = if typ.nullable {
                quote! { Option<#field_type_ident> }
            } else {
                quote! { #field_type_ident }
            };

            ProcessTypeResult {
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

            let field_type_ident = format_ident! {"List"};
            let name = format_ident! {"{name}"};

            let (inner_nullable, field_type_tokens) =
                match t.to_string().matches('!').count() {
                    0 => (false, quote! { Vec<#name> }),
                    1 => {
                        if t.to_string().ends_with('!') {
                            (true, quote! { Vec<Option<#name>> })
                        } else {
                            (false, quote! { Option<Vec<#name>> })
                        }
                    }
                    2 => (true, quote! { Option<Vec<Option<#name>>>> }),
                    _ => panic!("Invalid `FieldDefinition` type: {:?}.", t),
                };

            ProcessTypeResult {
                field_type_ident,
                field_type_tokens,
                base_type: FieldBaseType::Named,
                nullable: typ.nullable,
                inner_nullable,
                inner_type_ident: Some(format_ident! {"{name}"}),
            }
        }
    }
}

/// Return `FieldKind` for a given `FieldDefinition` within the context of a
/// particularly parsed GraphQL schema.
pub fn field_kind(
    field_typ_name: &str,
    fid: &str,
    parsed: &ParsedGraphQLSchema,
) -> FieldKind {
    // println!(">> ftyp name: {}", field_typ_name);
    // println!(">> LIST TYPES PARSED: {:?}", parsed.list_field_types);
    // let field_typ_name = inner_typ_name.map(|x| x.to_string()).unwrap_or(field_typ_name);
    // if !parsed.has_type(field_typ_name) {
    //     panic!("This is not a type: {:?}", field_typ_name);
    // }

    if parsed.is_list_field_type(fid) {
        // println!(">> LIST TYP NAME: {}", field_typ_name);
        let kind = if parsed.is_possible_foreign_key(field_typ_name) {
            FieldKind::ForeignKey
        } else {
            FieldKind::Scalar
        };
        return FieldKind::List(Box::new(kind));
    }
    if parsed.is_union_typedef(field_typ_name)
        && !parsed.is_possible_foreign_key(field_typ_name)
    {
        return FieldKind::Union;
    }

    if parsed.is_possible_foreign_key(field_typ_name) {
        return FieldKind::ForeignKey;
    }

    if parsed.is_enum_typedef(field_typ_name) {
        return FieldKind::Enum;
    }

    if parsed.is_virtual_typedef(field_typ_name) {
        return FieldKind::Virtual;
    }

    FieldKind::Scalar
}

/// Get tokens for a field's `.clone()`.
pub fn clone_tokens(field_typ_name: &str) -> TokenStream {
    if COPY_TYPES.contains(field_typ_name) {
        quote! {.clone()}
    } else {
        quote! {}
    }
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

/// Get tokens for a field's `.to_bytes()`.
pub fn to_bytes_tokens(field_typ_name: &str) -> TokenStream {
    if EXTERNAL_FIELD_TYPES.contains(field_typ_name) {
        to_bytes_method_for_external_type(field_typ_name)
    } else if !ASREF_BYTE_TYPES.contains(field_typ_name) {
        quote! { .to_le_bytes() }
    } else {
        quote! {}
    }
}

/// Get tokens for hasher.
pub fn hasher_tokens(
    field_type_scalar_name: &str,
    hasher: TokenStream,
    field_name: &Ident,
    clone: TokenStream,
    unwrap_or_default: TokenStream,
    to_bytes: TokenStream,
) -> Option<TokenStream> {
    if !NON_DIGESTIBLE_FIELD_TYPES.contains(field_type_scalar_name) {
        return Some(
            quote! { #hasher.chain_update(#field_name #clone #unwrap_or_default #to_bytes) },
        );
    }
    None
}

/// Get tokens for parameters used in `::new()` function and `::get_or_create()`
/// function/method signatures.
pub fn parameters_tokens(
    parameters: TokenStream,
    field_name: &Ident,
    typ_tokens: TokenStream,
) -> TokenStream {
    quote! { #parameters #field_name: #typ_tokens, }
}

/// Get tokens for a field decoder.
pub fn field_decoder_tokens(
    nullable: bool,
    field_type_scalar_name: &str,
    field_name: &Ident,
    clone: TokenStream,
) -> TokenStream {
    let field_type_scalar_name = format_ident! {"{}", field_type_scalar_name};
    if nullable {
        quote! { FtColumn::#field_type_scalar_name(self.#field_name #clone), }
    } else {
        quote! { FtColumn::#field_type_scalar_name(Some(self.#field_name #clone)), }
    }
}

/// Whether a given field is elligible for auto ID.
pub fn can_derive_id(
    field_set: &HashSet<String>,
    field_name: &str,
    typedef_name: &str,
) -> bool {
    field_set.contains(IdCol::to_lowercase_str())
        && !INTERNAL_INDEXER_ENTITIES.contains(typedef_name)
        && field_name != IdCol::to_lowercase_str()
}

// pub fn nullable_field_type_name(f: &FieldDefinition, field_name: &str) -> String {
//     let is_list_ty = f.ty.node.to_string().matches('[').count() > 0; // naive
//     match f.ty.node.to_string().matches('!').count() {
//         0 => if is_list_ty {
//             return format!("[{field_name}]");
//         } else {
//             return format!("{field_name}");
//         }
//         1 => {
//             if is_list_ty {
//                 if f.ty.node.to_string().ends_with('!') {
//                     return format!("[{field_name}]!");
//                 } else {
//                     return format!("[{field_name}!]");
//                 }
//             } else {
//                 return format!("{field_name}!");
//             }
//         }
//         2 => format!("[{field_name}!]!"),
//         _ => panic!("Invalid `FieldDefinition` type: {:?}.", f.ty.node),
//     }
// }
