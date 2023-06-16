use std::collections::HashSet;

use crate::constant::*;
use async_graphql_parser::types::UnionType;
use fuel_abi_types::abi::program::{ProgramABI, TypeDeclaration};
use fuel_indexer_database_types::IdCol;
use fuel_indexer_schema::parser::ParsedGraphQLSchema;
use fuels_code_gen::utils::Source;
use lazy_static::lazy_static;
use quote::{format_ident, quote};
use syn::Ident;

lazy_static! {
    /// Set of internal indexer entities.
    pub static ref INTERNAL_INDEXER_ENTITIES: HashSet<&'static str> = HashSet::from([
        "IndexMetadataEntity",
    ]);

    /// Set of types that implement `AsRef<[u8]>`.
    pub static ref ASREF_BYTE_TYPES: HashSet<&'static str> = HashSet::from([
        "Address",
        "AssetId",
        "Blob",
        "BlockId",
        "Boolean",
        "Bytes",
        "Bytes20",
        "Bytes32",
        "Bytes4",
        "Bytes64",
        "Bytes8",
        "Charfield",
        "ContractId",
        "HexString",
        "Json",
        "MessageId",
        "Virtual",
        "Nonce",
        "Option<Address>",
        "Option<AssetId>",
        "Option<Blob>",
        "Option<BlockId>",
        "Option<Boolean>",
        "Option<Bytes20>",
        "Option<Bytes32>",
        "Option<Bytes4>",
        "Option<Bytes64>",
        "Option<Bytes8>",
        "Option<Bytes>",
        "Option<Charfield>",
        "Option<ContractId>",
        "Option<HexString>",
        "Option<Json>",
        "Option<MessageId>",
        "Option<Virtual>",
        "Option<Nonce>",
        "Option<Salt>",
        "Option<Signature>",
        "Option<TxId>",
        "Salt",
        "Signature",
        "TxId",
    ]);

    /// Set of external types that do not implement `AsRef<[u8]>`.
    pub static ref EXTERNAL_FIELD_TYPES: HashSet<&'static str> = HashSet::from([
        "Identity",
        "Option<Identity>",
        "Option<Tai64Timestamp>",
        "Tai64Timestamp",
    ]);

    /// Set of field types that are currently unable to be used as a digest for SHA-256 hashing.
    pub static ref NONDIGESTIBLE_FIELD_TYPES: HashSet<&'static str> = HashSet::from([
        "Boolean",
        "Identity"
    ]);
}

/// Parameters for generating traits for different `TypeKind` variants.
pub enum TraitGenerationParameters<'a> {
    ObjectType {
        strct: Ident,
        parameters: proc_macro2::TokenStream,
        hasher: proc_macro2::TokenStream,
        object_name: String,
        struct_fields: proc_macro2::TokenStream,
        is_native: bool,
        field_set: HashSet<&'a String>,
    },
    UnionType {
        schema: &'a ParsedGraphQLSchema,
        union_obj: &'a UnionType,
        union_ident: Ident,
        union_field_set: HashSet<String>,
    },
}

/// Provides a TokenStream to be used as a conversion to bytes
/// for external types; this is done because traits cannot be
/// implemented on external types due to the orphan rule.
pub fn to_bytes_method_for_external_type(
    field_type_name: String,
) -> proc_macro2::TokenStream {
    match field_type_name.as_str() {
        "Identity" => quote! { .0 },
        "Tai64Timestamp" => quote! { .0.to_le_bytes() },
        _ => panic!("From<{field_type_name}> not implemented for AsRef<u8>."),
    }
}

/// Provides a TokenStream to be used for unwrapping `Option`s
/// for external types; this is done because traits cannot be
/// implemented on external types due to the orphan rule.
pub fn unwrap_or_default_for_external_type(
    field_type_name: String,
) -> proc_macro2::TokenStream {
    match field_type_name.as_str() {
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
    is_nullable: bool,
) -> proc_macro2::TokenStream {
    let type_name = field_type.to_string();
    if schema.is_enum_type(&type_name) {
        field_type = format_ident! {"UInt1"};
    }

    if is_nullable {
        quote! {
            let item = vec.pop().expect("Missing item in row.");
            let #field_name = match item {
                FtColumn::#field_type(t) => t,
                _ => panic!("Invalid nullable column type: {:?}.", item),
            };
        }
    } else {
        quote! {
            let item = vec.pop().expect("Missing item in row.");
            let #field_name = match item {
                FtColumn::#field_type(t) => match t {
                    Some(inner_type) => { inner_type },
                    None => {
                        panic!("Non-nullable type is returning a None value.")
                    }
                },
                _ => panic!("Invalid non-nullable column type: {:?}.", item),
            };
        }
    }
}

#[allow(clippy::too_many_arguments)]
/// Given a set of idents and tokens, construct the `Entity` and `Json` implementations
/// for the given struct.
pub fn generate_object_trait_impls(
    strct: Ident,
    strct_fields: proc_macro2::TokenStream,
    type_id: i64,
    field_extractors: proc_macro2::TokenStream,
    from_row: proc_macro2::TokenStream,
    to_row: proc_macro2::TokenStream,
    is_native: bool,
    trait_gen_params: TraitGenerationParameters,
) -> proc_macro2::TokenStream {
    let json_impl = quote! {

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
    };

    let entity_impl = if is_native {
        quote! {
            #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
            pub struct #strct {
                #strct_fields
            }

            #[async_trait::async_trait]
            impl Entity for #strct {
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
        }
    } else {
        quote! {
            #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
            pub struct #strct {
                #strct_fields
            }

            impl Entity for #strct {
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

        }
    };

    let instantiation_impls = match trait_gen_params {
        TraitGenerationParameters::ObjectType {
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
        TraitGenerationParameters::UnionType {
            schema,
            union_obj,
            union_ident,
            union_field_set,
        } => generate_from_traits_for_union(
            schema,
            union_obj,
            union_ident,
            union_field_set,
        ),
    };

    quote! {
        #entity_impl

        #instantiation_impls

        #json_impl
    }
}

/// Construct a `::new()` method for a particular struct; `::new()`
/// will automatically create an ID for the user for use in a database.
pub fn generate_struct_new_method_impl(
    strct: Ident,
    parameters: proc_macro2::TokenStream,
    hasher: proc_macro2::TokenStream,
    object_name: String,
    struct_fields: proc_macro2::TokenStream,
    is_native: bool,
) -> proc_macro2::TokenStream {
    let get_or_create_impl = if is_native {
        quote! {
            pub async fn get_or_create(self) -> Self {
                match Self::load(self.id).await {
                    Some(instance) => instance,
                    None => self,
                }
            }
        }
    } else {
        quote! {
            pub fn get_or_create(self) -> Self {
                match Self::load(self.id) {
                    Some(instance) => instance,
                    None => self,
                }
            }
        }
    };

    if !INTERNAL_INDEXER_ENTITIES.contains(object_name.as_str()) {
        quote! {
            impl #strct {
                pub fn new(#parameters) -> Self {
                    let raw_bytes = #hasher.chain_update(#object_name).finalize();

                    let id_bytes = <[u8; 8]>::try_from(&raw_bytes[..8]).expect("Could not calculate bytes for ID from struct fields");

                    let id = u64::from_le_bytes(id_bytes);

                    Self {
                        id,
                        #struct_fields
                    }
                }

                #get_or_create_impl
            }
        }
    } else {
        quote! {}
    }
}

/// Generate `From` trait implementations for each member type in a union.
pub fn generate_from_traits_for_union(
    schema: &ParsedGraphQLSchema,
    union_obj: &UnionType,
    union_ident: Ident,
    union_field_set: HashSet<String>,
) -> proc_macro2::TokenStream {
    let mut from_method_impls = quote! {};
    for m in union_obj.members.iter() {
        let member_ident = format_ident!("{}", m.to_string());

        let member_fields = schema
            .object_field_mappings
            .get(&m.to_string())
            .unwrap_or_else(|| {
                panic!(
                "Could not get field mappings for union member; union: {}, member: {}",
                union_ident,
                m
            )
            })
            .keys()
            .fold(HashSet::new(), |mut set, f| {
                set.insert(f.clone());
                set
            });

        // Member fields that match with union fields are checked for optionality
        // and are assigned accordingly.
        let common_fields = union_field_set.intersection(&member_fields).fold(
            quote! {},
            |acc, common_field| {
                let ident = format_ident!("{}", common_field);
                if common_field == &IdCol::to_lowercase_string() {
                    quote! {
                        #acc
                        #ident: member.#ident,
                    }
                } else if let Some(field_already_option) = schema
                    .field_type_optionality
                    .get(&format!("{m}.{common_field}"))
                {
                    if *field_already_option {
                        quote! {
                            #acc
                            #ident: member.#ident,
                        }
                    } else {
                        quote! {
                            #acc
                            #ident: Some(member.#ident),
                        }
                    }
                } else {
                    quote! { #acc }
                }
            },
        );

        // Any member fields that don't have a match with union fields should be assigned to None.
        let disjoint_fields = union_field_set.difference(&member_fields).fold(
            quote! {},
            |acc, disjoint_field| {
                let ident = format_ident!("{}", disjoint_field);
                quote! {
                    #acc
                    #ident: None,
                }
            },
        );

        from_method_impls = quote! {
            #from_method_impls

            impl From<#member_ident> for #union_ident {
                fn from(member: #member_ident) -> Self {
                    Self {
                        #common_fields
                        #disjoint_fields
                    }
                }
            }
        };
    }

    from_method_impls
}
