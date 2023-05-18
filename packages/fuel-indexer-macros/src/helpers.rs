use crate::constant::*;
use async_graphql_parser::parse_schema;
use async_graphql_parser::types::ServiceDocument;
use fuel_abi_types::program_abi::{ProgramABI, TypeDeclaration};
use fuel_indexer_schema::utils::{
    build_schema_fields_and_types_map, build_schema_types_set, BASE_SCHEMA,
};
use fuels_code_gen::utils::Source;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
use syn::Ident;

/// A wrapper object used to encapsulate a lot of the boilerplate logic related
/// to parsing schema, creating mappings of types, fields, objects, etc.
pub struct Schema {
    /// Namespace of the indexer.
    pub namespace: String,

    /// Identifier of the indexer.
    pub identifier: String,

    /// Whether we're building schema for native execution.
    pub is_native: bool,

    /// All unique names of types in the schema (whether objects, enums, or scalars).
    pub type_names: HashSet<String>,

    /// All unique names of enums in the schema.
    pub enum_names: HashSet<String>,

    /// All unique names of types for which tables should _not_ be created.
    pub non_indexable_type_names: HashSet<String>,

    /// All unique names of types that have already been parsed.
    pub parsed_type_names: HashSet<String>,

    /// All unique names of types that are possible foreign keys.
    pub foreign_key_names: HashSet<String>,

    /// A mapping of fully qualitified field names to their field types.
    pub field_type_mappings: HashMap<String, String>,

    /// All unique names of scalar types in the schema.
    pub scalar_names: HashSet<String>,
}

impl Schema {
    /// Create a new Schema.
    pub fn new(
        namespace: &str,
        identifier: &str,
        is_native: bool,
        ast: &ServiceDocument,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let base_ast = match parse_schema(BASE_SCHEMA) {
            Ok(ast) => ast,
            Err(e) => {
                proc_macro_error::abort_call_site!("Error parsing graphql schema {:?}", e)
            }
        };

        let (mut type_names, _) = build_schema_types_set(ast);
        let (scalar_names, _) = build_schema_types_set(&base_ast);
        type_names.extend(scalar_names.clone());

        Ok(Self {
            namespace: namespace.to_string(),
            identifier: identifier.to_string(),
            is_native,
            type_names,
            enum_names: HashSet::new(),
            non_indexable_type_names: HashSet::new(),
            parsed_type_names: HashSet::new(),
            foreign_key_names: HashSet::new(),
            field_type_mappings: build_schema_fields_and_types_map(ast)?,
            scalar_names,
        })
    }

    /// Whether the schema has a scalar type with the given name.
    pub fn has_scalar(&self, name: &str) -> bool {
        self.scalar_names.contains(name)
    }

    /// Whether the given field type name is a possible foreign key.
    pub fn is_possible_foreign_key(&self, name: &str) -> bool {
        self.parsed_type_names.contains(name) && !self.has_scalar(name)
    }

    /// Whether the given field type name is a type from which tables are created.
    #[allow(unused)]
    pub fn is_non_indexable_type(&self, name: &str) -> bool {
        self.non_indexable_type_names.contains(name)
    }

    pub fn is_enum_type(&self, name: &str) -> bool {
        self.enum_names.contains(name)
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
            "Revert" => quote! { abi::Revert },
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

pub fn row_extractor(
    schema: &Schema,
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
                _ => panic!("Invalid column type: {:?}.", item),
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
                _ => panic!("Invalid column type: {:?}.", item),
            };
        }
    }
}
