use crate::constant::*;
use fuel_abi_types::program_abi::{
    ABIFunction, LoggedType, ProgramABI, TypeApplication, TypeDeclaration,
};
use fuels_code_gen::utils::Source;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{
    AngleBracketedGenericArguments, GenericArgument, Ident,
    PathArguments::AngleBracketed, PathSegment, Type,
};

/// If TypeDeclaration is tuple type
pub fn is_tuple_type(typ: &TypeDeclaration) -> bool {
    typ.type_field.as_str().starts_with("(_")
}

/// If TypeDeclaration is a generic Vec type
#[allow(unused)]
pub fn is_vec_type(typ: &TypeDeclaration) -> bool {
    typ.type_field.as_str().starts_with("struct Vec")
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

/// Whether we should ignore this type
pub fn is_non_cacheable_type(typ: &TypeDeclaration) -> bool {
    if IGNORED_ABI_JSON_TYPES.contains(typ.type_field.as_str()) {
        return true;
    }

    if is_tuple_type(typ) {
        return true;
    }

    false
}

/// Whether the TypeDeclaration should be used to build struct fields
/// and decoders
pub fn is_non_parsable_type(typ: &TypeDeclaration) -> bool {
    if VEC_GENERIC_TYPES.contains(typ.type_field.as_str()) {
        return true;
    }

    if is_non_cacheable_type(typ) {
        return true;
    }

    false
}

/// Whether a path is a path for a vector
pub fn path_is_vec_ident(p: &str) -> bool {
    p == "Vec"
}

pub fn derive_vec_token(vec: &str, generic: &str) -> proc_macro2::TokenStream {
    let v = format_ident!("{}", vec);
    let g = format_ident!("{}", generic);
    quote! { #v<#g> }
}

/// Return path String and type ID for given generic PathSegment
pub fn vec_path_idents(p: &PathSegment) -> (String, Ident) {
    let v = p.ident.to_string();
    let generic = match &p.arguments {
        AngleBracketed(AngleBracketedGenericArguments { args, .. }) => {
            match args.last().unwrap() {
                GenericArgument::Type(t) => match t {
                    Type::Path(p) => {
                        let zero = p.path.segments.last().unwrap();
                        zero.ident.to_string()
                    }
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            }
        }
        _ => unimplemented!(),
    };

    let generic_ident = format_ident!("{}", generic);
    let generic_vec_tok = derive_vec_token(&v, &generic);
    let generic_vec_str = generic_vec_tok.to_string();

    (generic_vec_str, generic_ident)
}

/// Derive Ident for decoded type
pub fn decoded_ident(ty: &str) -> Ident {
    format_ident! { "{}_decoded", ty.to_ascii_lowercase() }
}

/// Return type field name for complex type
fn derive_complex_type_field(ty: &TypeDeclaration) -> String {
    ty.type_field
        .split(' ')
        .last()
        .expect("Could not parse TypeDeclaration for Rust name.")
        .to_string()
}

/// Equivalent of `decoded_ident` but for Vec type specifically
pub fn vec_decoded_ident(ty: &TypeDeclaration) -> Ident {
    let name = derive_complex_type_field(ty);
    let name = decoded_ident(&name);
    format_ident!("vec_{}", name.to_string())
}

/// Derive Ident for given TypeDeclaration
pub fn rust_ident(ty: &TypeDeclaration) -> Ident {
    if ty.components.is_some() {
        if is_tuple_type(ty) {
            proc_macro_error::abort_call_site!("Cannot derive rust_ident of tuple type.");
        }

        let name = derive_complex_type_field(ty);
        decoded_ident(&name)
    } else {
        let name = ty.type_field.replace(['[', ']'], "_");
        decoded_ident(&name)
    }
}

/// Given a set of ABI functions, logged types, and abi types, build a mapping
/// of Vec types to their generic types: where Vec<T> is { TypeId(Vec) -> TypeApplication(T) }
pub fn build_vec_generics(
    functions: &Vec<ABIFunction>,
    log_types: &HashMap<usize, LoggedType>,
    _abi_types: &HashMap<usize, TypeDeclaration>,
) -> HashMap<usize, Vec<TypeApplication>> {
    let mut results = HashMap::default();
    for func in functions {
        for input in func.inputs.iter() {
            if let Some(args) = &input.type_arguments {
                if let Some(last) = args.last() {
                    results
                        .entry(input.type_id)
                        .or_insert(Vec::new())
                        .push(last.clone());
                }
            }
        }
    }

    for (_ty_id, log_ty) in log_types.iter() {
        if let Some(args) = &log_ty.application.type_arguments {
            if let Some(last) = args.last() {
                results
                    .entry(log_ty.application.type_id)
                    .or_insert(Vec::new())
                    .push(last.clone());
            }
        }
    }

    results
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
            "Identity" => quote! { abi::Identity },
            "Log" => quote! { abi::Log },
            "LogData" => quote! { abi::LogData },
            "MessageOut" => quote! { abi::MessageOut },
            "Return" => quote! { abi::Return },
            "ScriptResult" => quote! { abi::ScriptResult },
            "Transfer" => quote! { abi::Transfer },
            "TransferOut" => quote! { abi::TransferOut },
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

/// Whether or not the given token is a Rust or Fuel primitive
pub fn is_primitive(ty: &proc_macro2::TokenStream) -> bool {
    is_rust_primitive(ty) || is_fuel_primitive(ty)
}

/// Whether an Ident is for a vector of types
pub fn is_vec_ident(ty: &Ident) -> bool {
    ty.to_string().as_str().starts_with("vec_")
}

/// Given a type ID, a type token, and a type Ident, return a decoder snippet
/// as a set of tokens
pub fn decode_snippet(
    ty_id: usize,
    ty: &proc_macro2::TokenStream,
    name: &Ident,
) -> proc_macro2::TokenStream {
    if is_primitive(ty) {
        quote! {
            #ty_id => {
                Logger::warn("Skipping primitive decoder.");
            }
        }
    } else if is_vec_ident(name) {
        quote! {
            #ty_id => {
                Logger::warn("Vec type decoder not implemented.");
                unimplemented!();
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
