use crate::helpers::*;
use async_graphql_parser::types::{
    FieldDefinition, ObjectType, TypeDefinition, TypeKind,
};
use async_graphql_parser::{Pos, Positioned};
use async_graphql_value::Name;
use fuel_indexer_lib::graphql::{
    check_for_directive, field_id, types::IdCol, ParsedGraphQLSchema,
    MAX_FOREIGN_KEY_LIST_FIELDS,
};
use fuel_indexer_types::type_id;
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

    /// Token stream used to generate `T::struct_field()` `Field<F>` or
    /// `OptionField<F>` which are the used to construct `Constraint<T>` for use
    /// with `T::find()`
    pub field_selectors: Vec<TokenStream>,

    /// `TypeDefinition`.
    typdef: TypeDefinition,

    /// The parsed GraphQL schema.
    //
    // Since `From<ImplementationDecoder> for TokenStream` uses `ParsedGraphQLSchema` to lookup
    // the fields of each member in the union, we need to include it here.
    parsed: ParsedGraphQLSchema,
}

impl Decoder for ImplementationDecoder {
    /// Create a decoder from a GraphQL `TypeDefinition`.
    fn from_typedef(typ: &TypeDefinition, parsed: &ParsedGraphQLSchema) -> Self {
        match &typ.kind {
            TypeKind::Object(o) => {
                // Remove any fields that use an internal type
                let obj_name = typ.name.to_string();

                let mut struct_fields = quote! {};
                let mut field_selectors = vec![];
                let mut parameters = quote! {};
                let mut hasher = quote! { Sha256::new() };

                let obj_field_names = parsed
                    .object_field_mappings()
                    .get(&obj_name)
                    .unwrap_or_else(|| panic!("TypeDefinition '{obj_name}' not found in parsed GraphQL schema."))
                    .iter()
                    .map(|(k, _v)| k.to_owned())
                    .collect::<HashSet<String>>();

                for field in &o.fields {
                    if check_for_directive(&field.node.directives, "internal") {
                        continue;
                    }

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

                    if can_derive_id(&obj_field_names, field_name) {
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

                    let ident = format_ident!("{obj_name}");
                    let field_name_ident_string = field_name_ident.to_string();

                    // Skip generics
                    if processed_type_result.inner_type_ident.is_none()
                        && !processed_type_result.nullable
                    {
                        field_selectors.push(quote! {
                            pub fn #field_name_ident () -> Field<#ident, #field_type_tokens> {
                                Field::new(#field_name_ident_string .to_string())
                            }
                        });
                    }

                    if processed_type_result.nullable
                        && processed_type_result.field_type_ident != "Array"
                    {
                        field_selectors.push(quote! {
                            pub fn #field_name_ident () -> OptionField<#ident, #field_type_ident> {
                                OptionField::new(#field_name_ident_string .to_string())
                            }
                        });
                    }
                }

                ImplementationDecoder {
                    parameters,
                    hasher,
                    struct_fields,
                    field_selectors,
                    typdef: typ.clone(),
                    parsed: parsed.clone(),
                }
            }
            TypeKind::Union(u) => {
                let union_name = typ.name.to_string();
                // Manually keep track of fields we've seen so we don't duplicate them.
                //
                // Other crates like `LinkedHashSet` preserve order but in a different way
                // than what is needed here.
                let mut seen_fields = HashSet::new();

                let fields = u
                    .members
                    .iter()
                    .flat_map(|m| {
                        // We grab the object `TypeDefinition` from the parsed schema so as to maintain the
                        // same order of the fields as they appear when being parsed in `ParsedGraphQLSchema`.
                        let name = m.node.to_string();
                        let mut fields = parsed
                            .object_ordered_fields()
                            .get(&name)
                            .expect("Could not find union member in parsed schema.")
                            .to_owned();

                        fields.sort_by(|a, b| a.1.cmp(&b.1));

                        fields
                            .iter()
                            // Remove any fields that are marked for internal use
                            .filter_map(|f| {
                                if check_for_directive(&f.0.directives, "internal") {
                                    return None;
                                }

                                Some(f.0.name.to_string())
                            })
                            .collect::<Vec<String>>()
                    })
                    .filter_map(|field_name| {
                        if seen_fields.contains(&field_name) {
                            return None;
                        }

                        seen_fields.insert(field_name.clone());

                        let field_id = field_id(&union_name, &field_name);
                        let f = &parsed
                            .field_defs()
                            .get(&field_id)
                            .expect("FieldDefinition not found in parsed schema.");
                        // All fields in a derived union type are nullable, except for the `ID` field.
                        let mut f = f.0.clone();
                        f.ty.node.nullable =
                            f.name.to_string() != IdCol::to_lowercase_str();
                        Some(Positioned {
                            pos: Pos::default(),
                            node: f,
                        })
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
            typdef,
            parsed,
            field_selectors,
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

        let impl_get_or_create = quote! {
            pub fn get_or_create(self) -> Self {
                match Self::load(self.id.clone()) {
                    Some(instance) => instance,
                    None => {
                        self.save();
                        self
                    },
                }
            }
        };

        match &typdef.kind {
            TypeKind::Object(o) => {
                let field_set = o
                    .fields
                    .iter()
                    .map(|f| f.node.name.to_string())
                    .collect::<HashSet<String>>();

                if !field_set.contains(IdCol::to_lowercase_str()) {
                    return quote! {};
                }

                // We allow for `clippy::too_many_arguments` here as an entity can
                // have a large number of fields and a method that uses those fields
                // as parameters will trigger this clippy lint.
                quote! {
                    impl #ident {
                        #[allow(clippy::too_many_arguments)]
                        pub fn new(#parameters) -> Self {
                            let hashed = #hasher.chain_update(#typdef_name).finalize();
                            let id = UID::new(format!("{:x}", hashed)).expect("Bad ID.");
                            Self {
                                id,
                                #struct_fields
                            }
                        }

                        #(#field_selectors)*

                        #impl_get_or_create
                    }
                }
            }

            TypeKind::Union(u) => {
                let union_name = typdef.name.to_string();

                let mut from_method_impls = quote! {};
                let get_or_create_tokens = if parsed.is_virtual_typedef(&typdef_name) {
                    quote! {}
                } else {
                    quote! {
                        impl #ident {
                            #(#field_selectors)*

                            #impl_get_or_create
                        }
                    }
                };

                // Manually keep track of fields we've seen so we don't duplicate them.
                //
                // Other crates like `LinkedHashSet` preserve order but in a different way
                // than what is needed here.
                let mut seen_fields = HashSet::new();

                let field_set = u
                    .members
                    .iter()
                    .flat_map(|m| {
                        // We grab the object `TypeDefinition` from the parsed schema so as to maintain the
                        // same order of the fields as they appear when being parsed in `ParsedGraphQLSchema`.
                        let name = m.node.to_string();
                        let mut fields = parsed
                            .object_ordered_fields()
                            .get(&name)
                            .expect("Could not find union member in parsed schema.")
                            .to_owned();

                        fields.sort_by(|a, b| a.1.cmp(&b.1));

                        fields
                            .iter()
                            // Remove any fields marked for internal use
                            .filter_map(|f| {
                                if check_for_directive(&f.0.directives, "internal") {
                                    return None;
                                }

                                Some(f.0.name.to_string())
                            })
                            .collect::<Vec<String>>()
                    })
                    .filter_map(|field_name| {
                        if seen_fields.contains(&field_name) {
                            return None;
                        }

                        seen_fields.insert(field_name.clone());

                        let field_id = field_id(&union_name, &field_name);
                        let f = &parsed
                            .field_defs()
                            .get(&field_id)
                            .expect("FieldDefinition not found in parsed schema.");
                        // All fields in a derived union type are nullable, except for the `ID` field.
                        let mut f = f.0.clone();
                        f.ty.node.nullable =
                            f.name.to_string() != IdCol::to_lowercase_str();
                        Some(Positioned {
                            pos: Pos::default(),
                            node: f,
                        })
                    })
                    .map(|f| f.node.name.to_string())
                    .collect::<Vec<String>>();

                let field_hashset: HashSet<String> =
                    HashSet::from_iter(field_set.iter().map(|f| f.to_owned()));

                u.members.iter().for_each(|m| {
                    let member_ident = format_ident!("{}", m.to_string());

                    let member_fields = parsed
                        .object_field_mappings()
                        .get(m.to_string().as_str())
                        .expect("Could not get field mappings for union member.")
                        .keys()
                        .map(|k| k.to_owned())
                        .collect::<HashSet<_>>();

                    // Member fields that match with union fields are checked for optionality
                    // and are assigned accordingly.
                    let common_fields = field_hashset.intersection(&member_fields).fold(
                        quote! {},
                        |acc, common_field| {
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
                        },
                    );

                    // Any member fields that don't have a match with union fields should be assigned to None.
                    let disjoint_fields = field_hashset.difference(&member_fields).fold(
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

                quote! {
                    #from_method_impls

                    #get_or_create_tokens
                }
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

    /// The unique ID of this GraphQL type.
    type_id: i64,
}

impl Decoder for ObjectDecoder {
    /// Create a decoder from a GraphQL `TypeDefinition`.
    fn from_typedef(typ: &TypeDefinition, parsed: &ParsedGraphQLSchema) -> Self {
        match &typ.kind {
            TypeKind::Object(o) => {
                let obj_name = typ.name.to_string();

                let ident = format_ident!("{obj_name}");
                let type_id = type_id(&parsed.fully_qualified_namespace(), &obj_name);

                let mut struct_fields = quote! {};
                let mut field_extractors = quote! {};
                let mut from_row = quote! {};
                let mut to_row = quote! {};

                let mut fields_map = BTreeMap::new();

                for field in o.fields.iter() {
                    if check_for_directive(&field.node.directives, "internal") {
                        continue;
                    }

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
                    impl_decoder: ImplementationDecoder::from_typedef(typ, parsed),
                    type_id,
                }
            }
            TypeKind::Union(u) => {
                let union_name = typ.name.to_string();

                // Manually keep track of fields we've seen so we don't duplicate them.
                //
                // Other crates like `LinkedHashSet` preserve order but in a different way
                // than what is needed here.
                let mut seen_fields = HashSet::new();

                let fields = u
                    .members
                    .iter()
                    .flat_map(|m| {
                        // We grab the object `TypeDefinition` from the parsed schema so as to maintain the
                        // same order of the fields as they appear when being parsed in `ParsedGraphQLSchema`.
                        let name = m.node.to_string();
                        let mut fields = parsed
                            .object_ordered_fields()
                            .get(&name)
                            .expect("Could not find union member in parsed schema.")
                            .to_owned();

                        fields.sort_by(|a, b| a.1.cmp(&b.1));

                        fields
                            .iter()
                            .map(|f| f.0.name.to_string())
                            .collect::<Vec<String>>()
                    })
                    .filter_map(|field_name| {
                        if seen_fields.contains(&field_name) {
                            return None;
                        }

                        seen_fields.insert(field_name.clone());

                        let field_id = field_id(&union_name, &field_name);
                        let f = &parsed
                            .field_defs()
                            .get(&field_id)
                            .expect("FieldDefinition not found in parsed schema.");
                        // All fields in a derived union type are nullable, except for the `ID` field.
                        let mut f = f.0.clone();
                        f.ty.node.nullable =
                            f.name.to_string() != IdCol::to_lowercase_str();
                        Some(Positioned {
                            pos: Pos::default(),
                            node: f,
                        })
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
            type_id,
            ..
        } = decoder;

        let impl_json = quote! {

            impl From<#ident> for Json {
                fn from(value: #ident) -> Self {
                    let s = serde_json::to_string(&value).expect("Failed to serialize Entity.");
                    Self::new(s)
                }
            }

            impl From<Json> for #ident {
                fn from(value: Json) -> Self {
                    let s: #ident = serde_json::from_str(&value.into_inner()).expect("Failed to deserialize Entity.");
                    s
                }
            }
        };

        let join_metadata = if let Some(meta) = impl_decoder
            .parsed
            .join_table_meta()
            .get(&ident.to_string())
        {
            let mut tokens =
                meta.iter()
                    .map(|meta| {
                        let table_name = meta.table_name();
                        let fully_qualified_namespace =
                            impl_decoder.parsed.fully_qualified_namespace();
                        let parent_column_name = meta.parent_column_name();
                        let child_column_name = meta.child_column_name();
                        let child_position = meta.parent().child_position.expect(
                            "Parent `JoinTableMeta` is missing `child_position`.",
                        );

                        quote! {
                            Some(JoinMetadata {
                                namespace: #fully_qualified_namespace,
                                table_name: #table_name,
                                parent_column_name: #parent_column_name,
                                child_column_name: #child_column_name,
                                child_position: #child_position,
                            })
                        }
                    })
                    .collect::<Vec<TokenStream>>();

            tokens.resize(MAX_FOREIGN_KEY_LIST_FIELDS, quote! { None });

            quote! {
                Some([ #( #tokens ),* ])
            }
        } else {
            quote! { None }
        };

        let impl_entity = quote! {
            #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
            pub struct #ident {
                #struct_fields
            }

            impl<'a> Entity<'a> for #ident {
                const TYPE_ID: i64 = #type_id;
                const JOIN_METADATA: Option<[Option<JoinMetadata<'a>>; MAX_FOREIGN_KEY_LIST_FIELDS]> = #join_metadata;

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
    use async_graphql_parser::types::{BaseType, ConstDirective, ObjectType, Type};
    use fuel_indexer_lib::graphql::GraphQLSchema;

    #[test]
    fn test_can_create_object_decoder_containing_expected_tokens_from_object_typedef() {
        let schema = r#"
type Person @entity {
    id: ID!
    name: String!
    age: U8!
}"#;

        let fields = [("id", "ID"), ("name", "String"), ("age", "U8")]
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
            directives: vec![Positioned {
                pos: Pos::default(),
                node: ConstDirective {
                    name: Positioned {
                        pos: Pos::default(),
                        node: Name::new("entity"),
                    },
                    arguments: vec![],
                },
            }],
        };

        let schema = ParsedGraphQLSchema::new(
            "test",
            "test",
            Some(&GraphQLSchema::new(schema.to_string())),
        )
        .unwrap();

        let decoder = ObjectDecoder::from_typedef(&typdef, &schema);
        let tokenstream = TokenStream::from(decoder).to_string();

        // Trying to assert we have every single token expected might be a bit much, so
        // let's just assert that we have the main/primary method and function definitions.
        assert!(tokenstream.contains("pub struct Person"));
        assert!(tokenstream.contains("impl < 'a > Entity < 'a > for Person"));
        assert!(tokenstream.contains("impl Person"));
        assert!(tokenstream.contains("pub fn new (name : String , age : U8 ,) -> Self"));
        assert!(tokenstream.contains("pub fn get_or_create (self) -> Self"));
        assert!(tokenstream.contains("fn from_row (mut vec : Vec < FtColumn >) -> Self"));
        assert!(tokenstream.contains("fn to_row (& self) -> Vec < FtColumn >"));
    }

    #[test]
    fn test_can_create_object_decoder_containing_expected_tokens_from_object_typedef_containing_m2m_relationship(
    ) {
        let schema = r#"
type Account @entity {
    id: ID!
    index: U64!
}

type Wallet @entity {
    id: ID!
    account: [Account!]!
}
"#;

        let fields = vec![
            Positioned {
                pos: Pos::default(),
                node: FieldDefinition {
                    description: None,
                    name: Positioned {
                        pos: Pos::default(),
                        node: Name::new("id"),
                    },
                    arguments: vec![],
                    ty: Positioned {
                        pos: Pos::default(),
                        node: Type {
                            base: BaseType::Named(Name::new("ID")),
                            nullable: false,
                        },
                    },
                    directives: vec![],
                },
            },
            Positioned {
                pos: Pos::default(),
                node: FieldDefinition {
                    description: None,
                    name: Positioned {
                        pos: Pos::default(),
                        node: Name::new("account"),
                    },
                    arguments: vec![],
                    ty: Positioned {
                        pos: Pos::default(),
                        node: Type {
                            base: BaseType::List(Box::new(Type {
                                base: BaseType::Named(Name::new("Account")),
                                nullable: false,
                            })),
                            nullable: false,
                        },
                    },
                    directives: vec![],
                },
            },
        ];

        let typdef = TypeDefinition {
            description: None,
            extend: false,
            name: Positioned {
                pos: Pos::default(),
                node: Name::new("Wallet"),
            },
            kind: TypeKind::Object(ObjectType {
                implements: vec![],
                fields,
            }),
            directives: vec![Positioned {
                pos: Pos::default(),
                node: ConstDirective {
                    name: Positioned {
                        pos: Pos::default(),
                        node: Name::new("entity"),
                    },
                    arguments: vec![],
                },
            }],
        };

        let schema = ParsedGraphQLSchema::new(
            "test",
            "test",
            Some(&GraphQLSchema::new(schema.to_string())),
        )
        .unwrap();

        let wallet_decoder = ObjectDecoder::from_typedef(&typdef, &schema);
        let tokenstream = TokenStream::from(wallet_decoder).to_string();

        // Trying to assert we have every single token expected might be a bit much, so
        // let's just assert that we have the main/primary method and function definitions.
        assert!(tokenstream.contains("const JOIN_METADATA : Option < [Option < JoinMetadata < 'a >> ; MAX_FOREIGN_KEY_LIST_FIELDS] > = Some ([Some (JoinMetadata { namespace : \"test_test\" , table_name : \"wallets_accounts\" , parent_column_name : \"id\" , child_column_name : \"id\" , child_position : 1usize , }) , None , None , None , None , None , None , None , None , None]) ;"));
    }
}
