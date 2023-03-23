use crate::constant::*;
use fuel_abi_types::program_abi::{ProgramABI, TypeDeclaration};
use fuels_code_gen::utils::Source;
use quote::{format_ident, quote};
use syn::Ident;

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
            "Call" => quote! { abi::Call },
            "Identity" => quote! { abi::Identity },
            "Log" => quote! { abi::Log },
            "LogData" => quote! { abi::LogData },
            "MessageOut" => quote! { abi::MessageOut },
            "Return" => quote! { abi::Return },
            "ScriptResult" => quote! { abi::ScriptResult },
            "Transfer" => quote! { abi::Transfer },
            "TransferOut" => quote! { abi::TransferOut },
            "Panic" => quote! { abi::Panic },
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
        || FUEL_PRIMITIVES_NAMESPACED.contains(ident_str.as_str())
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
