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

/// Whether this Path token is a generic
pub fn path_is_generic_struct(p: &str) -> bool {
    // TODO: Add others as needed
    matches!(p, "Vec")
}

/// Given a generic struct and it's T, return the corresponding Ident
pub fn generic_path_tokens(generic: &str, gt: &str) -> proc_macro2::TokenStream {
    // TODO: Add others as needed
    match generic {
        "Vec" => {
            let generic = format_ident!("{}", generic);
            let gt = format_ident!("{}", gt);
            quote! { #generic<#gt> }
        }
        _ => unimplemented!(),
    }
}

/// Return the fully qualified path String and Ident for a given generic PathSegment
pub fn generic_path_ident_tokens(p: &PathSegment) -> (String, Ident) {
    // TODO: Implement as needed
    let g = p.ident.to_string();
    let gt = match g.as_ref() {
        "Vec" => match &p.arguments {
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
        },
        _ => unimplemented!(),
    };

    let ident = format_ident!("{}", gt);
    let tokens = generic_path_tokens(&g, &gt);

    (tokens.to_string(), ident)
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

/// Equivalent of `decoded_ident` but for generic type specifically
pub fn generic_decoded_ident(gt: &TypeDeclaration, generic: &TypeDeclaration) -> Ident {
    match generic.type_field.to_string().as_ref() {
        "struct Vec" => {
            let name = decoded_ident(&derive_type_field(gt));
            format_ident!("vec_{}", name.to_string())
        }
        _ => unimplemented!(),
    }
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

/// Given a set of ABI functions, logged types, and abi types, build a mapping of Vec
// types to their generic types: where Vec<T> is { TypeId(Vec) -> TypeApplication(T) }
pub fn build_generics(
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
            "Call" => quote! { abi::Call },
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
        || FUEL_PRIMITIVES_NAMESPACED.contains(ident_str.as_str())
}

/// Whether or not the given token is a Rust primitive
pub fn is_rust_primitive(ty: &proc_macro2::TokenStream) -> bool {
    let ident_str = ty.to_string();
    RUST_PRIMITIVES.contains(ident_str.as_str())
}

/// Whether or not this is a generic primitive
fn is_generic_primitive(ty: &proc_macro2::TokenStream) -> bool {
    // TODO: Add more as needed
    let ty = ty.to_string();
    matches!(ty.starts_with("Vec"), true)
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
    } else if is_generic_primitive(ty) {
        quote! {
            #ty_id => {
                Logger::warn("Generic type decoder not implemented.");
                unimplemented!();
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
