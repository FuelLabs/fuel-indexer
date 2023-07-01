use crate::{constants::*, helpers::*};
use async_graphql_parser::types::{
    FieldDefinition, ObjectType, TypeDefinition, TypeKind,
};
use async_graphql_parser::{Pos, Positioned};
use async_graphql_value::Name;
use fuel_indexer_lib::{
    graphql::{field_id, types::IdCol, GraphQLSchemaValidator, ParsedGraphQLSchema},
    type_id, ExecutionSource,
};
use linked_hash_set::LinkedHashSet;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, HashSet};
use syn::Ident;

/// `Decoder`s are responsible for transforming GraphQL `TypeDefinition`s into
/// token streams that can be used to generate Rust code for indexing types.
pub trait Decoder {
    /// Create a decoder from a GraphQL `TypeDefinition`.
    fn from_typedef(typ: &TypeDefinition, parsed: &ParsedGraphQLSchema) -> Self;
}

/// Similar to `{Object,Enum}Decoder`, but specifically used to derive `::new()` function and
/// `::get_or_create()` function.
#[derive(Debug)]
pub struct ImplementationDecoder {
    /// Token stream of params passed to `::new()`.
    parameters: proc_macro2::TokenStream,

    /// Token stream of hasher.
    hasher: proc_macro2::TokenStream,

    /// Token stream of struct fields.
    struct_fields: TokenStream,

    /// Execution source of indexer.
    exec_source: ExecutionSource,

    /// `TypeDefinition`.
    typdef: TypeDefinition,

    /// The parsed GraphQL schema.
    ///
    /// Since `From<ImplementationDecoder> for TokenStream` uses `ParsedGraphQLSchema` to lookup
    /// the fields of each member in the union, we need to include it here.
    parsed: ParsedGraphQLSchema,
}

impl Default for ImplementationDecoder {
    fn default() -> Self {
        Self {
            parameters: quote! {},
            hasher: quote! {},
            struct_fields: quote! {},
            exec_source: ExecutionSource::Wasm,
            typdef: TypeDefinition {
                description: None,
                extend: false,
                name: Positioned::new(Name::new(String::new()), Pos::default()),
                kind: TypeKind::Object(ObjectType {
                    implements: vec![],
                    fields: vec![],
                }),
                directives: vec![],
            },
            parsed: ParsedGraphQLSchema::default(),
        }
    }
}

impl Decoder for ImplementationDecoder {
    /// Create a decoder from a GraphQL `TypeDefinition`.
    fn from_typedef(typ: &TypeDefinition, parsed: &ParsedGraphQLSchema) -> Self {
        match &typ.kind {
            TypeKind::Object(o) => {
                let obj_name = typ.name.to_string();
                let mut struct_fields = quote! {};
                let mut parameters = quote! {};
                let mut hasher = quote! { Sha256::new() };

                let obj_field_names = parsed
                    .object_field_mappings
                    .get(&obj_name)
                    .expect("TypeDefinition not found in parsed GraphQL schema.")
                    .iter()
                    .map(|(k, _v)| k.to_owned())
                    .collect::<HashSet<String>>();

                for field in &o.fields {
                    let ProcessedTypedefField {
                        field_name_ident,
                        processed_type_result,
                        ..
                    } = process_typedef_field(parsed, field.node.clone());

                    let ProcessedFieldType {
                        field_type_tokens,
                        field_type_ident,
                        nullable,
                        base_type,
                        ..
                    } = &processed_type_result;

                    let field_typ_name = &field_type_ident.to_string();
                    let field_name = &field_name_ident.to_string();

                    let clone = clone_tokens(
                        field_typ_name,
                        &field_id(&obj_name, field_name),
                        parsed,
                    );
                    let unwrap_or_default =
                        unwrap_or_default_tokens(field_typ_name, *nullable);

                    let to_bytes =
                        to_bytes_tokens(field_typ_name, &processed_type_result);

                    if can_derive_id(&obj_field_names, field_name, &obj_name) {
                        parameters = parameters_tokens(
                            &parameters,
                            &field_name_ident,
                            field_type_tokens,
                        );
                        if let Some(tokens) = hasher_tokens(
                            field_typ_name,
                            field_name,
                            base_type,
                            &hasher,
                            &clone,
                            &unwrap_or_default,
                            &to_bytes,
                        ) {
                            hasher = tokens;
                        }

                        struct_fields = quote! {
                            #struct_fields
                            #field_name_ident,
                        };
                    }
                }

                ImplementationDecoder {
                    parameters,
                    hasher,
                    struct_fields,
                    exec_source: parsed.exec_source().clone(),
                    typdef: typ.clone(),
                    parsed: parsed.clone(),
                }
            }
            TypeKind::Union(u) => {
                let union_name = typ.name.to_string();
                let member_fields = u
                    .members
                    .iter()
                    .flat_map(|m| {
                        let name = m.node.to_string();
                        parsed
                            .object_field_mappings
                            .get(&name)
                            .expect("Could not find union member in parsed schema.")
                            .iter()
                            .map(|(k, v)| (k.to_owned(), v.to_owned()))
                    })
                    .collect::<LinkedHashSet<(String, String)>>()
                    .iter()
                    .map(|(k, _)| {
                        let fid = field_id(&union_name, k);
                        let f = &parsed
                            .field_defs()
                            .get(&fid)
                            .expect("FielDefinition not found in parsed schema.");
                        // All fields in a derived union type are nullable, except for
                        // the `ID` field.
                        let mut f = f.0.clone();
                        f.ty.node.nullable =
                            f.name.to_string() != IdCol::to_lowercase_str();
                        Positioned {
                            pos: Pos::default(),
                            node: f,
                        }
                    })
                    .collect::<Vec<Positioned<FieldDefinition>>>();

                let typdef = TypeDefinition {
                    description: None,
                    extend: false,
                    name: Positioned {
                        pos: Pos::default(),
                        node: Name::new(union_name),
                    },
                    kind: TypeKind::Object(ObjectType {
                        implements: vec![],
                        fields: member_fields,
                    }),
                    directives: vec![],
                };

                Self::from_typedef(&typdef, parsed)
            }
            _ => unimplemented!(
                "TypeDefinition does not support ::new() and ::get_or_create()."
            ),
        }
    }
}

impl From<ImplementationDecoder> for TokenStream {
    fn from(decoder: ImplementationDecoder) -> Self {
        let ImplementationDecoder {
            parameters,
            hasher,
            struct_fields,
            exec_source,
            typdef,
            parsed,
        } = decoder;

        let typdef_name = typdef.name.to_string();
        let ident = format_ident!("{}", typdef_name);

        // When processing `TypeDefinition::Union`, instead of duplicating a lot of logic
        // in the `Decoder`s, we just recursively call `Decoder::from_typedef`.
        //
        // This works fine, but means that we will only technically ever have a `TypeDefinition::ObjectType`
        // on `ImplementationDecoder` - which means the `TypeKind::Union` codepath below will never
        // be called.
        //
        // To prevent this, we manually look into our `ParsedGraphQLSchema` to see if the `TypeDefinition`
        // we're given is a union type, and if so, we replace this `TypeDefinition::Object`, with that
        // `TypeDefinition::Union`.
        let typdef = parsed.get_union(&typdef_name).unwrap_or(&typdef);

        match &typdef.kind {
            TypeKind::Object(o) => {
                let field_set = o
                    .fields
                    .iter()
                    .map(|f| f.node.name.to_string())
                    .collect::<HashSet<String>>();

                if INTERNAL_INDEXER_ENTITIES.contains(typdef_name.as_str()) {
                    return quote! {};
                }

                if !field_set.contains(IdCol::to_lowercase_str()) {
                    return quote! {};
                }

                let impl_get_or_create = match exec_source {
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

                quote! {
                    impl #ident {
                        pub fn new(#parameters) -> Self {
                            let raw_bytes = #hasher.chain_update(#typdef_name).finalize();

                            let id_bytes = <[u8; 8]>::try_from(&raw_bytes[..8]).expect("Could not calculate bytes for ID from struct fields");

                            let id = u64::from_le_bytes(id_bytes);

                            Self {
                                id,
                                #struct_fields
                            }
                        }

                        #impl_get_or_create
                    }
                }
            }
            TypeKind::Union(u) => {
                let mut from_method_impls = quote! {};

                let union_field_set = u
                    .members
                    .iter()
                    .flat_map(|m| {
                        let name = m.node.to_string();
                        parsed
                            .object_field_mappings
                            .get(&name)
                            .expect("Could not find union member in parsed schema.")
                            .iter()
                            .map(|(k, v)| (k.to_owned(), v.to_owned()))
                    })
                    .collect::<LinkedHashSet<(String, String)>>()
                    .iter()
                    .map(|(k, _v)| k.to_owned())
                    .collect::<HashSet<String>>();

                u.members.iter().for_each(|m| {
                    let member_ident = format_ident!("{}", m.to_string());

                    let member_fields = parsed
                        .object_field_mappings
                        .get(m.to_string().as_str())
                        .expect("Could not get field mappings for union member.")
                        .keys()
                        .map(|k| k.to_owned())
                        .collect::<HashSet<String>>();

                    // Member fields that match with union fields are checked for optionality
                    // and are assigned accordingly.
                    let common_fields = union_field_set
                        .intersection(&member_fields)
                        .fold(quote! {}, |acc, common_field| {
                            let ident = format_ident!("{}", common_field);
                            let fid = field_id(&m.node, common_field);
                            if common_field == &IdCol::to_lowercase_string() {
                                quote! {
                                    #acc
                                    #ident: member.#ident,
                                }
                            } else if let Some(field_already_option) =
                                parsed.field_type_optionality().get(&fid)
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
                        });

                    // Any member fields that don't have a match with union fields should be assigned to None.
                    let disjoint_fields = union_field_set
                        .difference(&member_fields)
                        .fold(quote! {}, |acc, disjoint_field| {
                            let ident = format_ident!("{}", disjoint_field);
                            quote! {
                                #acc
                                #ident: None,
                            }
                        });

                    from_method_impls = quote! {
                        #from_method_impls

                        impl From<#member_ident> for #ident {
                            fn from(member: #member_ident) -> Self {
                                Self {
                                    #common_fields
                                    #disjoint_fields
                                }
                            }
                        }
                    };
                });

                from_method_impls
            }
            _ => unimplemented!("Cannot do this with enum."),
        }
    }
}

/// A wrapper object used to process GraphQL `TypeKind::Object` type definitions
/// into a format from which Rust tokens can be generated.
pub struct ObjectDecoder {
    /// The name of the GraphQL object (as a `syn::Ident`).
    ident: Ident,

    /// Tokens used to create fields in the struct definition.
    struct_fields: TokenStream,

    /// Tokens used to extract each individual field from a row.
    field_extractors: TokenStream,

    /// Tokens used to create fields in the `Entity::from_row` function.
    from_row: TokenStream,

    /// Tokens used to create fields in the `Entity::to_row` function.
    to_row: TokenStream,

    /// Tokens for the parameters of the `Entity::new` function.
    impl_decoder: ImplementationDecoder,

    /// The source of the GraphQL schema.
    exec_source: ExecutionSource,

    /// The unique ID of this GraphQL type.
    type_id: i64,
}

impl Default for ObjectDecoder {
    fn default() -> Self {
        Self {
            ident: format_ident!("unnamed"),
            struct_fields: quote! {},
            field_extractors: quote! {},
            from_row: quote! {},
            to_row: quote! {},
            exec_source: ExecutionSource::Wasm,
            impl_decoder: ImplementationDecoder::default(),
            type_id: std::i64::MAX,
        }
    }
}

impl Decoder for ObjectDecoder {
    /// Create a decoder from a GraphQL `TypeDefinition`.
    fn from_typedef(typ: &TypeDefinition, parsed: &ParsedGraphQLSchema) -> Self {
        match &typ.kind {
            TypeKind::Object(o) => {
                let obj_name = typ.name.to_string();

                GraphQLSchemaValidator::check_disallowed_graphql_typedef_name(&obj_name);

                let ident = format_ident!("{obj_name}");
                let type_id = type_id(&parsed.fully_qualified_namespace(), &obj_name);

                let mut struct_fields = quote! {};
                let mut field_extractors = quote! {};
                let mut from_row = quote! {};
                let mut to_row = quote! {};

                let mut fields_map = BTreeMap::new();

                for field in &o.fields {
                    let ProcessedTypedefField {
                        field_name_ident,
                        extractor,
                        processed_type_result,
                    } = process_typedef_field(parsed, field.node.clone());

                    let ProcessedFieldType {
                        field_type_tokens,
                        field_type_ident,
                        ..
                    } = &processed_type_result;

                    fields_map.insert(
                        field_name_ident.to_string(),
                        field_type_ident.to_string(),
                    );

                    let clone = clone_tokens(
                        &field_type_ident.to_string(),
                        &field_id(&obj_name, &field_name_ident.to_string()),
                        parsed,
                    );
                    let field_decoder = field_decoder_tokens(
                        &field_name_ident,
                        &clone,
                        &processed_type_result,
                    );

                    struct_fields = quote! {
                        #struct_fields
                        #field_name_ident: #field_type_tokens,
                    };

                    field_extractors = quote! {
                        #extractor
                        #field_extractors
                    };

                    from_row = quote! {
                        #from_row
                        #field_name_ident,
                    };

                    to_row = quote! {
                        #to_row
                        #field_decoder
                    };
                }

                Self {
                    ident,
                    struct_fields,
                    field_extractors,
                    from_row,
                    to_row,
                    exec_source: parsed.exec_source().clone(),
                    impl_decoder: ImplementationDecoder::from_typedef(typ, parsed),
                    type_id,
                }
            }
            TypeKind::Union(u) => {
                let union_name = typ.name.to_string();
                let fields = u
                    .members
                    .iter()
                    .flat_map(|m| {
                        let name = m.node.to_string();
                        parsed
                            .object_field_mappings
                            .get(&name)
                            .expect("Could not find union member in parsed schema.")
                            .iter()
                            .map(|(k, v)| (k.to_owned(), v.to_owned()))
                    })
                    .collect::<LinkedHashSet<(String, String)>>()
                    .iter()
                    .map(|(k, _)| {
                        let fid = field_id(&union_name, k);
                        let f = &parsed
                            .field_defs()
                            .get(&fid)
                            .expect("FieldDefinition not found in parsed schema.");
                        // All fields in a derived union type are nullable, except for
                        // the `ID` field.
                        let mut f = f.0.clone();
                        f.ty.node.nullable =
                            f.name.to_string() != IdCol::to_lowercase_str();
                        Positioned {
                            pos: Pos::default(),
                            node: f,
                        }
                    })
                    .collect::<Vec<Positioned<FieldDefinition>>>();

                let typdef = TypeDefinition {
                    description: None,
                    extend: false,
                    name: Positioned {
                        pos: Pos::default(),
                        node: Name::new(union_name),
                    },
                    kind: TypeKind::Object(ObjectType {
                        implements: vec![],
                        fields,
                    }),
                    directives: vec![],
                };

                Self::from_typedef(&typdef, parsed)
            }
            _ => panic!("Expected `TypeKind::Union` or `TypeKind::Object."),
        }
    }
}

/// A wrapper object used to process GraphQL `TypeKind::Enum` type definitions
/// into a format from which Rust tokens can be generated.
pub struct EnumDecoder {
    /// The name of the GraphQL enum (as a `syn::Ident`).
    ident: Ident,

    /// Tokens used to create fields in the `From<String> for #ident` function.
    to_enum: Vec<proc_macro2::TokenStream>,

    /// Tokens used to create fields in the `From<#ident> for String` function.
    from_enum: Vec<proc_macro2::TokenStream>,

    /// Tokens used to create fields in the enum definition.
    values: Vec<TokenStream>,

    /// The unique ID of this GraphQL type.
    ///
    /// Type IDs for enum types are only for reference since an enum is a virtual type.
    #[allow(unused)]
    type_id: i64,
}

impl Decoder for EnumDecoder {
    /// Create a decoder from a GraphQL `TypeDefinition`.
    fn from_typedef(typ: &TypeDefinition, parsed: &ParsedGraphQLSchema) -> Self {
        match &typ.kind {
            TypeKind::Enum(e) => {
                let enum_name = typ.name.to_string();
                let ident = format_ident!("{enum_name}");
                let type_id = type_id(&parsed.fully_qualified_namespace(), &enum_name);

                let values = e
                    .values
                    .iter()
                    .map(|v| {
                        let ident = format_ident! {"{}", v.node.value.to_string()};
                        quote! { #ident }
                    })
                    .collect::<Vec<TokenStream>>();

                let to_enum = e
                    .values
                    .iter()
                    .map(|v| {
                        let value_ident = format_ident! {"{}", v.node.value.to_string()};
                        let as_str = format!("{ident}::{value_ident}");
                        quote! { #as_str => #ident::#value_ident, }
                    })
                    .collect::<Vec<proc_macro2::TokenStream>>();

                let from_enum = e
                    .values
                    .iter()
                    .map(|v| {
                        let value_ident = format_ident! {"{}", v.node.value.to_string()};
                        let as_str = format!("{ident}::{value_ident}");
                        quote! { #ident::#value_ident => #as_str.to_string(), }
                    })
                    .collect::<Vec<proc_macro2::TokenStream>>();

                Self {
                    ident,
                    to_enum,
                    from_enum,
                    values,
                    type_id,
                }
            }
            _ => panic!("Expected `TypeKind::Enum`."),
        }
    }
}

impl From<ObjectDecoder> for TokenStream {
    fn from(decoder: ObjectDecoder) -> Self {
        let ObjectDecoder {
            struct_fields,
            ident,
            field_extractors,
            from_row,
            to_row,
            impl_decoder,
            exec_source,
            type_id,
            ..
        } = decoder;

        let impl_json = quote! {

            impl From<#ident> for Json {
                fn from(value: #ident) -> Self {
                    let s = serde_json::to_string(&value).expect("Serde error.");
                    Self(s)
                }
            }

            impl From<Json> for #ident {
                fn from(value: Json) -> Self {
                    let s: #ident = serde_json::from_str(&value.0).expect("Serde error.");
                    s
                }
            }
        };

        let impl_entity = match exec_source {
            ExecutionSource::Native => quote! {
                #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
                pub struct #ident {
                    #struct_fields
                }

                #[async_trait::async_trait]
                impl Entity for #ident {
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
            },
            ExecutionSource::Wasm => quote! {
                #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
                pub struct #ident {
                    #struct_fields
                }

                impl Entity for #ident {
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

            },
        };

        let impl_new = TokenStream::from(impl_decoder);

        quote! {
            #impl_entity

            #impl_new

            #impl_json
        }
    }
}

impl From<EnumDecoder> for TokenStream {
    fn from(decoder: EnumDecoder) -> Self {
        let EnumDecoder {
            ident,
            to_enum,
            from_enum,
            values,
            ..
        } = decoder;

        quote! {
            #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
            pub enum #ident {
                #(#values),*
            }

            impl From<#ident> for String {
                fn from(val: #ident) -> Self {
                    match val {
                        #(#from_enum)*
                        _ => panic!("Unrecognized enum value."),
                    }
                }
            }

            impl From<String> for #ident {
                fn from(val: String) -> Self {
                    match val.as_ref() {
                        #(#to_enum)*
                        _ => panic!("Unrecognized enum value."),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use async_graphql_parser::types::{BaseType, ObjectType, Type};
    use fuel_indexer_lib::graphql::GraphQLSchema;

    #[test]
    fn test_can_create_object_decoder_containing_expected_tokens_from_object_typedef() {
        let schema = r#"
type Person {
    id: ID!
    name: Charfield!
    age: UInt1!
}"#;

        let fields = [("id", "ID"), ("name", "Charfield"), ("age", "UInt1")]
            .iter()
            .map(|(name, typ)| Positioned {
                pos: Pos::default(),
                node: FieldDefinition {
                    description: None,
                    name: Positioned {
                        pos: Pos::default(),
                        node: Name::new(name),
                    },
                    arguments: vec![],
                    ty: Positioned {
                        pos: Pos::default(),
                        node: Type {
                            base: BaseType::Named(Name::new(typ)),
                            nullable: false,
                        },
                    },
                    directives: vec![],
                },
            })
            .collect::<Vec<Positioned<FieldDefinition>>>();
        let typdef = TypeDefinition {
            description: None,
            extend: false,
            name: Positioned {
                pos: Pos::default(),
                node: Name::new("Person"),
            },
            kind: TypeKind::Object(ObjectType {
                implements: vec![],
                fields,
            }),
            directives: vec![],
        };

        let schema = ParsedGraphQLSchema::new(
            "test",
            "test",
            ExecutionSource::Wasm,
            Some(&GraphQLSchema::new(schema.to_string())),
        )
        .unwrap();

        let decoder = ObjectDecoder::from_typedef(&typdef, &schema);
        let tokenstream = TokenStream::from(decoder).to_string();

        // Trying to assert we have every single token expected might be a bit much, so
        // let's just assert that we have the main/primary method and function definitions.
        assert!(tokenstream.contains("pub struct Person"));
        assert!(tokenstream.contains("impl Entity for Person"));
        assert!(tokenstream.contains("impl Person"));
        assert!(
            tokenstream.contains("pub fn new (name : Charfield , age : UInt1 ,) -> Self")
        );
        assert!(tokenstream.contains("pub fn get_or_create (self) -> Self"));
        assert!(tokenstream.contains("fn from_row (mut vec : Vec < FtColumn >) -> Self"));
        assert!(tokenstream.contains("fn to_row (& self) -> Vec < FtColumn >"));
    }
}
