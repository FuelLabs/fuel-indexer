use std::collections::HashSet;

use crate::constant::*;
use fuel_abi_types::program_abi::{ProgramABI, TypeDeclaration};
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
        "Boolean",
        "Bytes32",
        "Bytes4",
        "Bytes64",
        "Bytes8",
        "Blob",
        "Charfield",
    ]);
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

pub fn generate_struct_new_method_impl(
    strct: Ident,
    parameters: proc_macro2::TokenStream,
    hasher: proc_macro2::TokenStream,
    object_name: String,
    struct_fields: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let new_method_impl = if !INTERNAL_INDEXER_ENTITIES.contains(object_name.as_str()) {
        quote! {
            impl #strct {
                pub fn new(#parameters) -> Self {
                    let raw_bytes = #hasher.chain_update(#object_name).finalize();

                    let id_bytes = <[u8; 8]>::try_from(&raw_bytes[..8]).expect("Could not calculate bytes for ID from struct fields");

                    let id = u64::from_be_bytes(id_bytes);

                    Self {
                        id,
                        #struct_fields
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    new_method_impl
}

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
            #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

    quote! {
        #entity_impl

        #json_impl
    }
}
