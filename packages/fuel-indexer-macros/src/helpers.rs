use std::collections::HashSet;

use crate::constants::*;
use async_graphql_parser::types::{BaseType, FieldDefinition, Type, UnionType};
use async_graphql_value::Name;
use fuel_abi_types::abi::program::{ProgramABI, TypeDeclaration};
use fuel_indexer_database_types::*;
use fuel_indexer_lib::{graphql::*, ExecutionSource};
use fuel_indexer_schema::utils::get_join_directive_info;
use fuel_indexer_types::type_id;
use fuels_code_gen::utils::Source;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

/// Parameters for generating traits for different `TypeKind` variants.
pub enum ImplNewParameters {
    ObjectType {
        strct: Ident,
        parameters: proc_macro2::TokenStream,
        hasher: proc_macro2::TokenStream,
        object_name: String,
        struct_fields: proc_macro2::TokenStream,
        exec_source: ExecutionSource,
        field_set: HashSet<String>,
    },
    // This is actually used, but it's implemented in a `TokenStream` which prevents
    // `clippy` from being able to find it, so ignore this lint.
    #[allow(unused)]
    UnionType {
        schema: ParsedGraphQLSchema,
        union_obj: UnionType,
        union_ident: Ident,
        union_field_set: HashSet<String>,
    },
}

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

/// Construct a `::new()` method for a particular struct; `::new()`
/// will automatically create an ID for the user for use in a database.
pub fn generate_struct_new_method_impl(
    ident: Ident,
    parameters: proc_macro2::TokenStream,
    hasher: proc_macro2::TokenStream,
    typedef_name: String,
    struct_fields: proc_macro2::TokenStream,
    exec_source: ExecutionSource,
) -> proc_macro2::TokenStream {
    let get_or_create_impl = match exec_source {
        ExecutionSource::Native => {
            quote! {
                pub async fn get_or_create(self) -> Self {
                    match Self::load(self.id).await {
                        Some(instance) => instance,
                        None => self,
                    }
                }
            }
        }
        ExecutionSource::Wasm => {
            quote! {
                pub fn get_or_create(self) -> Self {
                    match Self::load(self.id) {
                        Some(instance) => instance,
                        None => self,
                    }
                }
            }
        }
    };

    if !INTERNAL_INDEXER_ENTITIES.contains(typedef_name.as_str()) {
        quote! {
            impl #ident {
                pub fn new(#parameters) -> Self {
                    let raw_bytes = #hasher.chain_update(#typedef_name).finalize();
                    // let raw_bytes: [u8; 16] = [0u8; 16];

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
                    .field_type_optionality()
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

/// Derive a type ID for a `TypeKind` under a given indexer namespace and identifier.
pub fn typedef_type_id(namespace: &str, identifier: &str, typedef_name: &str) -> i64 {
    type_id(&format!("{namespace}_{identifier}"), typedef_name)
}

/// Type of special fields in GraphQL schema.
#[derive(Debug, Clone)]
pub enum FieldKind {
    ForeignKey,
    Enum,
    Virtual,
    Union,
    Regular,
}

/// Process an object's field and return a group of tokens.
///
/// This group of tokens include:
///     - The field's type tokens.
///     - The field's name as an Ident.
///     - The field's type as an Ident.
///     - The field's row extractor tokens.
pub fn process_typedef_field(
    schema: &ParsedGraphQLSchema,
    mut field_def: FieldDefinition,
    typedef_name: &String,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::Ident,
    proc_macro2::Ident,
    proc_macro2::TokenStream,
) {
    let field_name = field_def.name.to_string();
    let (typ_tokens, field_type_ident) = process_type(schema, &field_def.ty.node);
    let fieldkind = field_kind(&field_type_ident.to_string(), schema);

    match fieldkind {
        FieldKind::ForeignKey => {
            let directives::Join {
                reference_field_type_name,
                ..
            } = get_join_directive_info(
                &field_def,
                typedef_name,
                schema.field_type_mappings(),
            );

            let field_typ_name = nullable_type(&field_def, &reference_field_type_name);

            // We're manually updated the field type here because we need to substitute the field name
            // into a scalar type name.
            field_def.ty.node = Type {
                base: BaseType::Named(Name::new(normalize_field_type_name(
                    &field_typ_name,
                ))),
                nullable: field_def.ty.node.nullable,
            };

            process_typedef_field(schema, field_def, typedef_name)
        }
        FieldKind::Enum => {
            let field_typ_name = nullable_type(&field_def, "Charfield");
            field_def.ty.node = Type {
                base: BaseType::Named(Name::new(normalize_field_type_name(
                    &field_typ_name,
                ))),
                nullable: field_def.ty.node.nullable,
            };
            process_typedef_field(schema, field_def, typedef_name)
        }
        FieldKind::Virtual => {
            let field_typ_name = nullable_type(&field_def, "Virtual");
            field_def.ty.node = Type {
                base: BaseType::Named(Name::new(normalize_field_type_name(
                    &field_typ_name,
                ))),
                nullable: field_def.ty.node.nullable,
            };
            process_typedef_field(schema, field_def, typedef_name)
        }
        FieldKind::Union => {
            let field_typ_name = field_def.ty.to_string();
            match schema.is_virtual_type(&field_typ_name) {
                true => {
                    // All union derived type fields are optional.
                    field_def.ty.node = Type::new("Virtual").expect("Bad type.");
                    process_typedef_field(schema, field_def, typedef_name)
                }
                false => process_typedef_field(schema, field_def, typedef_name),
            }
        }
        _ => {
            let name_ident = format_ident! {"{field_name}"};
            let extractor = field_extractor(
                schema,
                name_ident.clone(),
                field_type_ident.clone(),
                field_def.ty.node.nullable,
            );

            (typ_tokens, name_ident, field_type_ident, extractor)
        }
    }
}

/// Process a named type into its type tokens, and the Ident for those type tokens.
pub fn process_type(
    schema: &ParsedGraphQLSchema,
    typ: &Type,
) -> (proc_macro2::TokenStream, proc_macro2::Ident) {
    match &typ.base {
        BaseType::Named(t) => {
            let name = t.to_string();
            if !schema.has_type(&name) {
                panic!("Type '{name}' is not defined in the schema.");
            }

            let name = format_ident! {"{}", name};

            if typ.nullable {
                (quote! { Option<#name> }, name)
            } else {
                (quote! { #name }, name)
            }
        }
        BaseType::List(_t) => panic!("Got a list type, we don't handle this yet..."),
    }
}

/// Return `FieldKind` for a given `FieldDefinition` within the context of a
/// particularly parsed GraphQL schema.
pub fn field_kind(field_typ_name: &str, parser: &ParsedGraphQLSchema) -> FieldKind {
    if parser.is_union_type(field_typ_name) {
        return FieldKind::Union;
    }

    if parser.is_possible_foreign_key(field_typ_name) {
        return FieldKind::ForeignKey;
    }

    if parser.is_enum_type(field_typ_name) {
        return FieldKind::Enum;
    }

    if parser.is_virtual_type(field_typ_name) {
        return FieldKind::Virtual;
    }

    FieldKind::Regular
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

/// Get tokens for parameters.
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

/// Return the GraphQL type with nullable suffix.
pub fn nullable_type(f: &FieldDefinition, field_name: &str) -> String {
    if f.ty.node.nullable {
        format!("{field_name}!")
    } else {
        field_name.to_string()
    }
}
