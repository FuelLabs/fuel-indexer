use fuel_abi_types::abi::program::TypeDeclaration;
use fuel_indexer_lib::constants::IGNORED_GENERIC_METADATA;

/// Strip the call path from the type field of a `TypeDeclaration`.
///
/// It is possible that the type field for a `TypeDeclaration` contains a
/// fully-qualified path (e.g. `std::address::Address` as opposed to `Address`).
/// Path separators are not allowed to be used as part of an identifier, so this
/// function removes the qualifying path while keeping the type keyword.
pub fn strip_callpath_from_type_field(mut typ: TypeDeclaration) -> TypeDeclaration {
    if is_non_decodable_type(&typ) {
        return typ;
    }

    let mut s = typ.type_field.split_whitespace();
    typ.type_field =
        if let (Some(keyword), Some(fully_qualified_type_path)) = (s.next(), s.last()) {
            if let Some(slug) = fully_qualified_type_path.split("::").last() {
                [keyword, slug].join(" ")
            } else {
                unreachable!("All types should be formed with a keyword and call path")
            }
        } else {
            typ.type_field
        };
    typ
}

/// Whether a `TypeDeclaration` is tuple type
pub fn is_tuple_type(typ: &TypeDeclaration) -> bool {
    let mut type_field_chars = typ.type_field.chars();
    type_field_chars.next().is_some_and(|c| c == '(')
        && type_field_chars.next().is_some_and(|c| c != ')')
}

/// Whether a `TypeDeclaration` is a unit type
pub fn is_unit_type(typ: &TypeDeclaration) -> bool {
    let mut type_field_chars = typ.type_field.chars();
    type_field_chars.next().is_some_and(|c| c == '(')
        && type_field_chars.next().is_some_and(|c| c == ')')
}

/// Whether the `TypeDeclaration` should be used to build struct fields and decoders
pub fn is_non_decodable_type(typ: &TypeDeclaration) -> bool {
    is_tuple_type(typ)
        || is_unit_type(typ)
        || IGNORED_GENERIC_METADATA.contains(typ.type_field.as_str())
}
