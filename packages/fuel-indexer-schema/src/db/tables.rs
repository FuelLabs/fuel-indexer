use crate::{
    db::IndexerSchemaDbResult, parser::ParsedGraphQLSchema, utils::*, QUERY_ROOT,
};
use async_graphql_parser::types::{
    BaseType, FieldDefinition, Type, TypeDefinition, TypeKind, TypeSystemDefinition,
};
use async_graphql_parser::{Pos, Positioned};
use async_graphql_value::Name;
use fuel_indexer_database::{
    queries, types::*, DbType, IndexerConnection, IndexerConnectionPool,
};
use fuel_indexer_types::type_id;
use linked_hash_set::LinkedHashSet;
use std::collections::{BTreeMap, HashMap, HashSet};

type CompleteProcessedType = (ColumnType, bool, Option<(ColumnType, bool, Option<Type>)>);

/// SchemaBuilder is used to encapsulate most of the logic related to parsing
/// GraphQL types, generating SQL from those types, and committing that SQL to
/// the database.
#[derive(Default)]
pub struct SchemaBuilder {
    db_type: DbType,
    statements: Vec<String>,
    type_ids: Vec<TypeId>,
    columns: Vec<NewColumn>,
    foreign_keys: Vec<ForeignKey>,
    indices: Vec<ColumnIndex>,
    namespace: String,
    identifier: String,
    version: String,
    schema: String,
    types: HashSet<String>,
    /// Schema field mapping is namespaced by type name
    fields: HashMap<String, HashMap<String, String>>,
    parsed_schema: ParsedGraphQLSchema,
    is_native: bool,
}

impl SchemaBuilder {
    /// Create a new `SchemaBuilder`.
    pub fn new(
        namespace: &str,
        identifier: &str,
        version: &str,
        db_type: DbType,
        is_native: bool,
    ) -> IndexerSchemaDbResult<SchemaBuilder> {
        let parsed_schema =
            ParsedGraphQLSchema::new(namespace, identifier, is_native, None)?;

        Ok(SchemaBuilder {
            db_type,
            namespace: namespace.to_string(),
            identifier: identifier.to_string(),
            version: version.to_string(),
            parsed_schema,
            is_native,
            ..Default::default()
        })
    }

    /// Generate table SQL for each object in the given schema.
    pub fn build(mut self, schema: &str) -> IndexerSchemaDbResult<Self> {
        if DbType::Postgres == self.db_type {
            let create = format!(
                "CREATE SCHEMA IF NOT EXISTS {}_{}",
                self.namespace, self.identifier
            );
            self.statements.push(create);
        }

        let parsed_schema = ParsedGraphQLSchema::new(
            &self.namespace,
            &self.identifier,
            self.is_native,
            Some(schema),
        )?;

        self.schema = schema.to_string();
        self.parsed_schema = parsed_schema.clone();

        for def in parsed_schema.ast.definitions.iter() {
            if let TypeSystemDefinition::Type(typ) = def {
                self.generate_table_sql(&typ.node)?;
            }
        }

        Ok(self)
    }

    /// Commit all SQL metadata to the database.
    pub async fn commit_metadata(
        self,
        conn: &mut IndexerConnection,
    ) -> IndexerSchemaDbResult<Schema> {
        let SchemaBuilder {
            version,
            statements,
            type_ids,
            columns,
            foreign_keys,
            indices,
            namespace,
            identifier,
            types,
            fields,
            schema,
            ..
        } = self;

        let new_root = NewGraphRoot {
            version: version.clone(),
            schema_name: namespace.clone(),
            schema_identifier: identifier.clone(),
            schema: schema.clone(),
        };
        queries::new_graph_root(conn, new_root).await?;

        let latest = queries::graph_root_latest(conn, &namespace, &identifier).await?;

        let type_names = fields.keys();

        let cols: Vec<_> = type_names
            .map(|t| NewRootColumns {
                root_id: latest.id,
                column_name: t.to_string(),
                graphql_type: t.to_string(),
            })
            .collect();

        queries::new_root_columns(conn, cols).await?;

        for query in statements {
            queries::execute_query(conn, query).await?;
        }

        for fk in foreign_keys {
            queries::execute_query(conn, fk.create_statement()).await?;
        }

        for idx in indices {
            queries::execute_query(conn, idx.create_statement()).await?;
        }

        queries::type_id_insert(conn, type_ids).await?;
        queries::new_column_insert(conn, columns).await?;

        let mut schema = Schema {
            version,
            namespace,
            identifier,
            types,
            fields,
            foreign_keys: HashMap::new(),
            non_indexable_types: HashSet::new(),
        };
        schema.register_queryroot_fields();

        Ok(schema)
    }

    /// Return a field's `ColumnType` and whether it is nullable.
    fn process_type(&self, ty: &Type) -> IndexerSchemaDbResult<CompleteProcessedType> {
        match &ty.base {
            BaseType::Named(t) => {
                if self.parsed_schema.is_enum_type(t.as_str()) {
                    return Ok((ColumnType::Charfield, false, None));
                }

                if self.parsed_schema.is_non_indexable_non_enum(t.as_str()) {
                    return Ok((ColumnType::Virtual, true, None));
                }

                if self.parsed_schema.is_possible_foreign_key(t.as_str()) {
                    return Ok((ColumnType::ForeignKey, ty.nullable, None));
                }

                Ok((ColumnType::from(t.as_str()), ty.nullable, None))
            }
            BaseType::List(t) => {
                let inner_type = self.process_list_inner_type(&t.clone())?;
                if inner_type.0 == ColumnType::ForeignKey {
                    Ok((ColumnType::ListComplex, ty.nullable, Some(inner_type)))
                } else {
                    Ok((ColumnType::ListScalar, ty.nullable, Some(inner_type)))
                }
            }
        }
    }

    fn process_list_inner_type(
        &self,
        ty: &Type,
    ) -> IndexerSchemaDbResult<(ColumnType, bool, Option<Type>)> {
        match &ty.base {
            BaseType::Named(t) => {
                if self.parsed_schema.is_enum_type(t.as_str()) {
                    return Ok((ColumnType::Charfield, false, None));
                }

                if self.parsed_schema.is_non_indexable_non_enum(t.as_str()) {
                    return Ok((ColumnType::Virtual, true, None));
                }

                if self.parsed_schema.is_possible_foreign_key(t.as_str()) {
                    return Ok((ColumnType::ForeignKey, ty.nullable, None));
                }

                Ok((ColumnType::from(t.as_str()), ty.nullable, None))
            }
            BaseType::List(_) => Err(super::IndexerSchemaDbError::ListOfListsUnsupported),
        }
    }

    /// Generate column SQL for each field in the given set of fields.
    fn generate_columns(
        &mut self,
        object_name: &String,
        type_id: i64,
        fields: &[FieldDefinition],
        table_name: &str,
    ) -> IndexerSchemaDbResult<String> {
        let mut fragments = Vec::new();

        for (pos, field) in fields.iter().enumerate() {
            let directives::Virtual(no_table) = get_notable_directive_info(field)?;
            if no_table {
                self.parsed_schema
                    .virtual_type_names
                    .insert(object_name.to_string());
            }

            let (typ, nullable, inner_list_typ) = self.process_type(&field.ty.node)?;

            let directives::Unique(unique) = get_unique_directive(field);

            match typ {
                ColumnType::ForeignKey => {
                    let directives::Join {
                        reference_field_name,
                        field_type_name,
                        reference_field_type_name,
                        ..
                    } = get_join_directive_info(
                        field,
                        object_name,
                        &self.parsed_schema.field_type_mappings,
                    );

                    let fk = ForeignKey::new(
                        self.db_type.clone(),
                        self.namespace(),
                        table_name.to_string(),
                        field.name.to_string(),
                        field_type_table_name(field),
                        reference_field_name.clone(),
                        reference_field_type_name.to_owned(),
                    );

                    let column = NewColumn {
                        type_id,
                        column_position: pos as i32,
                        column_name: field.name.to_string(),
                        column_type: reference_field_type_name.to_owned(),
                        graphql_type: field_type_name,
                        nullable,
                        unique,
                        is_list_with_nullable_elements: None,
                        inner_list_element_type: None,
                    };

                    fragments.push(column.sql_fragment());
                    self.columns.push(column);
                    self.foreign_keys.push(fk);
                }
                ColumnType::Virtual => {
                    let column = NewColumn {
                        type_id,
                        column_position: pos as i32,
                        column_name: field.name.to_string(),
                        column_type: typ.to_string(),
                        graphql_type: field.ty.to_string(),
                        nullable,
                        unique,
                        is_list_with_nullable_elements: None,
                        inner_list_element_type: None,
                    };

                    if let Some(directives::Index {
                        column_name,
                        method,
                    }) = get_index_directive(field)
                    {
                        self.indices.push(ColumnIndex {
                            db_type: self.db_type.clone(),
                            table_name: table_name.to_string(),
                            namespace: self.namespace(),
                            method,
                            unique,
                            column_name,
                        });
                    }

                    fragments.push(column.sql_fragment());
                    self.columns.push(column);
                }
                ColumnType::ListScalar => {
                    if let Some((element_col_type, has_nullable_elements, _)) =
                        inner_list_typ
                    {
                        match element_col_type {
                            // TODO: Do we want to allow these?
                            ColumnType::ID => todo!(),
                            ColumnType::Enum => todo!(),
                            ColumnType::Virtual => todo!(),
                            ColumnType::ListScalar | ColumnType::ListComplex => {
                                return Err(
                                    super::IndexerSchemaDbError::ListOfListsUnsupported,
                                );
                            }
                            _ => {
                                let column = NewColumn {
                                    type_id,
                                    column_position: pos as i32,
                                    column_name: field.name.to_string(),
                                    column_type: typ.to_string(),
                                    graphql_type: field.ty.to_string(),
                                    nullable,
                                    unique,
                                    is_list_with_nullable_elements: Some(
                                        has_nullable_elements,
                                    ),
                                    inner_list_element_type: Some(
                                        element_col_type.to_string(),
                                    ),
                                };

                                fragments.push(column.sql_fragment());
                                self.columns.push(column);
                            }
                        }
                    }
                }
                ColumnType::ListComplex => {
                    let directives::Join {
                        // reference_field_name,
                        // field_type_name,
                        reference_field_type_name,
                        ..
                    } = get_join_directive_info(
                        field,
                        object_name,
                        &self.parsed_schema.field_type_mappings,
                    );
                    if let Some((_element_col_type, has_nullable_elements, _)) =
                        inner_list_typ
                    {
                        let column = NewColumn {
                            type_id,
                            column_position: pos as i32,
                            column_name: field.name.to_string(),
                            column_type: "ListComplex".to_string(),
                            graphql_type: field.ty.to_string(),
                            nullable,
                            unique: false,
                            is_list_with_nullable_elements: Some(has_nullable_elements),
                            inner_list_element_type: Some(
                                reference_field_type_name.clone(),
                            ),
                        };
                        fragments.push(column.sql_fragment());
                        self.columns.push(column);
                    }
                }
                _ => {
                    let column = NewColumn {
                        type_id,
                        column_position: pos as i32,
                        column_name: field.name.to_string(),
                        column_type: typ.to_string(),
                        graphql_type: field.ty.to_string(),
                        nullable,
                        unique,
                        is_list_with_nullable_elements: None,
                        inner_list_element_type: None,
                    };

                    if let Some(directives::Index {
                        column_name,
                        method,
                    }) = get_index_directive(field)
                    {
                        self.indices.push(ColumnIndex {
                            db_type: self.db_type.clone(),
                            table_name: table_name.to_string(),
                            namespace: self.namespace(),
                            method,
                            unique,
                            column_name,
                        });
                    }

                    fragments.push(column.sql_fragment());
                    self.columns.push(column);
                }
            }
        }

        let object_column = NewColumn {
            type_id,
            column_position: fragments.len() as i32,
            column_name: "object".to_string(),
            column_type: "Object".to_string(),
            graphql_type: "__".into(),
            nullable: false,
            unique: false,
            is_list_with_nullable_elements: None,
            inner_list_element_type: None,
        };

        fragments.push(object_column.sql_fragment());
        self.columns.push(object_column);

        Ok(fragments.join(",\n"))
    }

    /// In SQL, we namespace a table by the schema name and the identifier, so
    /// as to allow for the same object names to be used across indexer namespaces.
    fn namespace(&self) -> String {
        format!("{}_{}", self.namespace, self.identifier)
    }

    /// Generate table SQL for a given type definition
    fn generate_table_sql(&mut self, typ: &TypeDefinition) -> IndexerSchemaDbResult<()> {
        match &typ.kind {
            TypeKind::Scalar => {}
            TypeKind::Enum(e) => {
                self.parsed_schema.enum_names.insert(typ.name.to_string());

                self.fields.insert(
                    typ.name.to_string(),
                    e.values
                        .iter()
                        .map(|v| (v.node.value.to_string(), "".to_string()))
                        .collect::<HashMap<String, String>>(),
                );

                let type_id = type_id(&self.namespace(), &typ.name.to_string());
                let table_name = typ.name.to_string().to_lowercase();
                self.type_ids.push(TypeId {
                    id: type_id,
                    schema_version: self.version.to_string(),
                    schema_name: self.namespace.to_string(),
                    schema_identifier: self.identifier.to_string(),
                    graphql_name: typ.name.to_string(),
                    table_name,
                    virtual_columns: e
                        .values
                        .iter()
                        .map(|v| VirtualColumn {
                            name: v.node.value.to_string(),
                            graphql_type: typ.name.to_string(),
                        })
                        .collect::<Vec<VirtualColumn>>(),
                });
            }
            TypeKind::Object(o) => {
                self.types.insert(typ.name.to_string());

                let mut fields_map = BTreeMap::new();

                self.parsed_schema
                    .parsed_type_names
                    .insert(typ.name.to_string());

                let field_defs: &[FieldDefinition] = &o
                    .fields
                    .iter()
                    .map(|f| {
                        fields_map.insert(f.node.name.to_string(), f.node.ty.to_string());
                        f.node.clone()
                    })
                    .collect::<Vec<FieldDefinition>>()[..];

                self.parsed_schema
                    .object_field_mappings
                    .insert(typ.name.to_string(), fields_map);

                self.fields.insert(
                    typ.name.to_string(),
                    field_defs
                        .iter()
                        .map(|f| (f.name.to_string(), f.ty.to_string()))
                        .collect(),
                );

                let table_name = typ.name.to_string().to_lowercase();
                let ty_id = type_id(&self.namespace(), &typ.name.to_string());
                let columns = self.generate_columns(
                    &typ.name.to_string(),
                    ty_id,
                    field_defs,
                    &table_name,
                )?;

                if self
                    .parsed_schema
                    .is_non_indexable_non_enum(typ.name.node.as_str())
                {
                    let ty_id = type_id(&self.namespace(), &typ.name.to_string());
                    let table_name = typ.name.to_string().to_lowercase();
                    self.type_ids.push(TypeId {
                        id: ty_id,
                        schema_version: self.version.to_string(),
                        schema_name: self.namespace.to_string(),
                        schema_identifier: self.identifier.to_string(),
                        graphql_name: typ.name.to_string(),
                        table_name,
                        virtual_columns: field_defs
                            .iter()
                            .map(|f| VirtualColumn {
                                name: f.name.to_string(),
                                graphql_type: f.ty.to_string(),
                            })
                            .collect::<Vec<VirtualColumn>>(),
                    });
                    return Ok(());
                }

                let sql_table = self.db_type.table_name(&self.namespace(), &table_name);

                let create =
                    format!("CREATE TABLE IF NOT EXISTS\n {sql_table} (\n {columns}\n)",);

                self.statements.push(create);
                self.type_ids.push(TypeId {
                    id: ty_id,
                    schema_version: self.version.to_string(),
                    schema_name: self.namespace.to_string(),
                    schema_identifier: self.identifier.to_string(),
                    graphql_name: typ.name.to_string(),
                    table_name,
                    virtual_columns: Vec::new(),
                });
            }
            TypeKind::Union(u) => {
                // We process this type effectively the same as we process `TypeKind::Object`.
                //
                // Except instead of using the `FieldDefinition` provided by the parser, we manually
                // construct the `FieldDefinition` based on the fields of the union members.
                self.parsed_schema.union_names.insert(typ.name.to_string());

                self.types.insert(typ.name.to_string());

                self.parsed_schema
                    .parsed_type_names
                    .insert(typ.name.to_string());

                let field_defs = u
                    .members
                    .iter()
                    .flat_map(|m| {
                        let name = m.node.to_string();
                        self.parsed_schema
                            .object_field_mappings
                            .get(&name)
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not find union member '{name}' in the schema."
                                );
                            })
                            .iter()
                            .map(|(k, v)| (k.to_owned(), v.to_owned()))
                    })
                    .collect::<LinkedHashSet<(String, String)>>()
                    .into_iter()
                    .map(|(k, v)| FieldDefinition {
                        description: None,
                        name: Positioned::new(Name::new(k), Pos::default()),
                        arguments: Vec::new(),
                        ty: Positioned::new(
                            Type {
                                base: BaseType::Named(Name::new(
                                    normalize_field_type_name(&v),
                                )),
                                nullable: v != *IdCol::to_uppercase_str(),
                            },
                            Pos::default(),
                        ),
                        directives: Vec::new(),
                    })
                    .collect::<Vec<FieldDefinition>>();

                self.fields.insert(
                    typ.name.to_string(),
                    field_defs
                        .iter()
                        .map(|f| (f.name.to_string(), f.ty.to_string()))
                        .collect(),
                );

                let table_name = typ.name.to_string().to_lowercase();
                let ty_id = type_id(&self.namespace(), &typ.name.to_string());
                let columns = self.generate_columns(
                    &typ.name.to_string(),
                    ty_id,
                    &field_defs,
                    &table_name,
                )?;

                let sql_table = self.db_type.table_name(&self.namespace(), &table_name);

                let create =
                    format!("CREATE TABLE IF NOT EXISTS\n {sql_table} (\n {columns}\n)",);

                self.statements.push(create);
                self.type_ids.push(TypeId {
                    id: ty_id,
                    schema_version: self.version.to_string(),
                    schema_name: self.namespace.to_string(),
                    schema_identifier: self.identifier.to_string(),
                    graphql_name: typ.name.to_string(),
                    table_name,
                    virtual_columns: Vec::new(),
                });
            }
            // TODO: Don't panic, return Err
            other_type => panic!("Parsed an unsupported type: '{other_type:?}'"),
        }

        Ok(())
    }
}

/// Schema is a `SchemaBuilder`-friendly representation of the parsed GraphQL schema
/// as it exists in the database.
#[derive(Debug, Clone)]
pub struct Schema {
    pub version: String,
    pub namespace: String,
    pub identifier: String,
    pub types: HashSet<String>,
    /// Schema field mapping is namespaced by type name
    pub fields: HashMap<String, HashMap<String, String>>,
    pub foreign_keys: HashMap<String, HashMap<String, (String, String)>>,
    pub non_indexable_types: HashSet<String>,
}

impl Schema {
    /// Load a `Schema` from the database.

    // TODO: Might be expensive to always load this from the DB each time. Maybe
    // we can cache and stash this somewhere?
    pub async fn load_from_db(
        pool: &IndexerConnectionPool,
        namespace: &str,
        identifier: &str,
    ) -> IndexerSchemaDbResult<Self> {
        let mut conn = pool.acquire().await?;
        let root = queries::graph_root_latest(&mut conn, namespace, identifier).await?;
        let typeids = queries::type_id_list_by_name(
            &mut conn,
            &root.schema_name,
            &root.version,
            identifier,
        )
        .await?;

        let mut types = HashSet::new();
        let mut non_indexable_types = HashSet::new();
        let mut fields = HashMap::new();

        for tyid in &typeids {
            types.insert(tyid.graphql_name.clone());

            if tyid.is_non_indexable_type() {
                let columns = tyid.virtual_columns.clone();
                fields.insert(
                    tyid.graphql_name.clone(),
                    columns
                        .into_iter()
                        .map(|c| (c.name, c.graphql_type))
                        .collect(),
                );
                non_indexable_types.insert(tyid.graphql_name.to_owned());
            } else {
                let columns = queries::list_column_by_id(&mut conn, tyid.id).await?;
                fields.insert(
                    tyid.graphql_name.to_owned(),
                    columns
                        .into_iter()
                        .map(|c| (c.column_name, c.graphql_type))
                        .collect(),
                );
            }
        }

        let foreign_keys = get_foreign_keys(namespace, identifier, false, &root.schema)?;

        let mut schema = Schema {
            version: root.version,
            namespace: root.schema_name,
            identifier: root.schema_identifier,
            types,
            fields,
            foreign_keys,
            non_indexable_types,
        };

        schema.register_queryroot_fields();

        Ok(schema)
    }

    /// Return the field type for a given field name on a given type name.
    pub fn field_type(&self, cond: &str, name: &str) -> Option<&String> {
        match self.fields.get(cond) {
            Some(fieldset) => fieldset.get(name),
            _ => {
                let tablename = normalize_field_type_name(cond);
                match self.fields.get(&tablename) {
                    Some(fieldset) => fieldset.get(name),
                    _ => None,
                }
            }
        }
    }

    /// Ensure the given type is included in this `Schema`'s types
    pub fn check_type(&self, type_name: &str) -> bool {
        self.types.contains(type_name)
    }

    // **** HACK ****

    // Below we manually add a `QueryRoot` type, with its corresponding field types
    // data being each `Object` defined in the schema.

    // We need this because at the moment our GraphQL query parsing is tightly-coupled
    // to our old way of resolving GraphQL types (which was using a `QueryType` object
    // defined in a `TypeSystemDefinition::Schema`)

    /// Register the `QueryRoot` type and its corresponding field types.
    pub fn register_queryroot_fields(&mut self) {
        self.fields.insert(
            QUERY_ROOT.to_string(),
            self.fields
                .keys()
                .map(|k| (k.to_lowercase(), k.clone()))
                .collect::<HashMap<String, String>>(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_builder_for_basic_postgres_schema_returns_proper_create_sql() {
        let graphql_schema: &str = r#"
        type Thing1 {
            id: ID!
            account: Address!
        }

        type Thing2 {
            id: ID!
            account: Address!
            hash: Bytes32!
        }
    "#;

        let create_schema: &str = "CREATE SCHEMA IF NOT EXISTS test_namespace_index1";
        let create_thing1_schmea: &str = concat!(
            "CREATE TABLE IF NOT EXISTS\n",
            " test_namespace_index1.thing1 (\n",
            " id numeric(20, 0) primary key not null,\n",
            "account varchar(64) not null,\n",
            "object bytea not null",
            "\n)"
        );
        let create_thing2_schema: &str = concat!(
            "CREATE TABLE IF NOT EXISTS\n",
            " test_namespace_index1.thing2 (\n",
            " id numeric(20, 0) primary key not null,\n",
            "account varchar(64) not null,\n",
            "hash varchar(64) not null,\n",
            "object bytea not null\n",
            ")"
        );

        let sb = SchemaBuilder::new(
            "test_namespace",
            "index1",
            "a_version_string",
            DbType::Postgres,
            false,
        );

        let SchemaBuilder { statements, .. } = sb.unwrap().build(graphql_schema).unwrap();

        assert_eq!(statements[0], create_schema);
        assert_eq!(statements[1], create_thing1_schmea);
        assert_eq!(statements[2], create_thing2_schema);
    }

    #[test]
    fn test_schema_builder_for_basic_postgres_schema_with_optional_types_returns_proper_create_sql(
    ) {
        let graphql_schema: &str = r#"
        type Thing1 {
            id: ID!
            account: Address
        }

        type Thing2 {
            id: ID!
            account: Address
            hash: Bytes32
        }
    "#;

        let create_schema: &str = "CREATE SCHEMA IF NOT EXISTS test_namespace_index1";
        let create_thing1_schmea: &str = concat!(
            "CREATE TABLE IF NOT EXISTS\n",
            " test_namespace_index1.thing1 (\n",
            " id numeric(20, 0) primary key not null,\n",
            "account varchar(64),\n",
            "object bytea not null",
            "\n)"
        );
        let create_thing2_schema: &str = concat!(
            "CREATE TABLE IF NOT EXISTS\n",
            " test_namespace_index1.thing2 (\n",
            " id numeric(20, 0) primary key not null,\n",
            "account varchar(64),\n",
            "hash varchar(64),\n",
            "object bytea not null\n",
            ")"
        );

        let sb = SchemaBuilder::new(
            "test_namespace",
            "index1",
            "a_version_string",
            DbType::Postgres,
            false,
        );

        let SchemaBuilder { statements, .. } = sb.unwrap().build(graphql_schema).unwrap();

        assert_eq!(statements[0], create_schema);
        assert_eq!(statements[1], create_thing1_schmea);
        assert_eq!(statements[2], create_thing2_schema);
    }

    #[test]
    fn test_schema_builder_for_postgres_indices_returns_proper_create_sql() {
        let graphql_schema: &str = r#"
        type Payer {
            id: ID!
            account: Address! @indexed
        }

        type Payee {
            id: ID!
            account: Address!
            hash: Bytes32! @indexed
        }
    "#;

        let sb = SchemaBuilder::new("namespace", "index1", "v1", DbType::Postgres, false);

        let SchemaBuilder { indices, .. } = sb.unwrap().build(graphql_schema).unwrap();

        assert_eq!(indices.len(), 2);
        assert_eq!(
            indices[0].create_statement(),
            "CREATE INDEX payer_account_idx ON namespace_index1.payer USING btree (account);"
                .to_string()
        );
        assert_eq!(
            indices[1].create_statement(),
            "CREATE INDEX payee_hash_idx ON namespace_index1.payee USING btree (hash);"
                .to_string()
        );
    }

    #[test]
    fn test_schema_builder_for_postgres_foreign_keys_returns_proper_create_sql() {
        let graphql_schema: &str = r#"
        type Borrower {
            id: ID!
            account: Address! @indexed
        }

        type Lender {
            id: ID!
            account: Address!
            hash: Bytes32! @indexed
            borrower: Borrower!
        }

        type Auditor {
            id: ID!
            account: Address!
            hash: Bytes32! @indexed
            borrower: Borrower!
        }
    "#;

        let sb = SchemaBuilder::new("namespace", "index1", "v1", DbType::Postgres, false);

        let SchemaBuilder { foreign_keys, .. } =
            sb.unwrap().build(graphql_schema).unwrap();

        assert_eq!(foreign_keys.len(), 2);
        assert_eq!(foreign_keys[0].create_statement(), "ALTER TABLE namespace_index1.lender ADD CONSTRAINT fk_lender_borrower__borrower_id FOREIGN KEY (borrower) REFERENCES namespace_index1.borrower(id) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
        assert_eq!(foreign_keys[1].create_statement(), "ALTER TABLE namespace_index1.auditor ADD CONSTRAINT fk_auditor_borrower__borrower_id FOREIGN KEY (borrower) REFERENCES namespace_index1.borrower(id) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
    }

    #[test]
    fn test_schema_builder_for_postgres_foreign_keys_with_directive_returns_proper_create_sql(
    ) {
        let graphql_schema: &str = r#"
        type Borrower {
            account: Address! @indexed
        }

        type Lender {
            id: ID!
            borrower: Borrower! @join(on:account)
        }

        type Auditor {
            id: ID!
            account: Address!
            hash: Bytes32! @indexed
            borrower: Borrower! @join(on:account)
        }
    "#;

        let sb = SchemaBuilder::new("namespace", "index1", "v1", DbType::Postgres, false);

        let SchemaBuilder { foreign_keys, .. } =
            sb.unwrap().build(graphql_schema).unwrap();

        assert_eq!(foreign_keys.len(), 2);
        assert_eq!(foreign_keys[0].create_statement(), "ALTER TABLE namespace_index1.lender ADD CONSTRAINT fk_lender_borrower__borrower_account FOREIGN KEY (borrower) REFERENCES namespace_index1.borrower(account) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
        assert_eq!(foreign_keys[1].create_statement(), "ALTER TABLE namespace_index1.auditor ADD CONSTRAINT fk_auditor_borrower__borrower_account FOREIGN KEY (borrower) REFERENCES namespace_index1.borrower(account) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
    }

    #[test]
    fn test_schema_builder_for_postgres_creates_fk_with_proper_column_names() {
        let graphql_schema: &str = r#"
        type Account {
            id: ID!
            account: Address! @indexed
        }

        type Message {
            id: ID!
            sender: Account!
            receiver: Account!
        }
    "#;

        let sb = SchemaBuilder::new("namespace", "index1", "v1", DbType::Postgres, false);

        let SchemaBuilder { foreign_keys, .. } =
            sb.unwrap().build(graphql_schema).unwrap();

        assert_eq!(foreign_keys.len(), 2);
        assert_eq!(foreign_keys[0].create_statement(), "ALTER TABLE namespace_index1.message ADD CONSTRAINT fk_message_sender__account_id FOREIGN KEY (sender) REFERENCES namespace_index1.account(id) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
        assert_eq!(foreign_keys[1].create_statement(), "ALTER TABLE namespace_index1.message ADD CONSTRAINT fk_message_receiver__account_id FOREIGN KEY (receiver) REFERENCES namespace_index1.account(id) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
    }

    #[test]
    fn test_get_implicit_foreign_keys_for_schema() {
        let implicit_schema: &str = r#"
        type Borrower {
            id: ID!
            account: Address! @indexed
        }

        type Lender {
            id: ID!
            account: Address!
            hash: Bytes32! @indexed
            borrower: Borrower!
        }

        type Auditor {
            id: ID!
            account: Address!
            hash: Bytes32! @indexed
            borrower: Borrower!
        }
    "#;

        let mut expected = HashMap::new();
        expected.insert(
            "lender".to_string(),
            HashMap::from([(
                "borrower".to_string(),
                ("borrower".to_string(), "id".to_string()),
            )]),
        );
        expected.insert(
            "auditor".to_string(),
            HashMap::from([(
                "borrower".to_string(),
                ("borrower".to_string(), "id".to_string()),
            )]),
        );

        let implicit_fks =
            get_foreign_keys("foo", "bar", false, implicit_schema).unwrap();
        assert_eq!(expected, implicit_fks);
    }

    #[test]
    fn test_get_explicit_foreign_keys_for_schema() {
        let explicit_schema: &str = r#"
        type Borrower {
            account: Address! @indexed
        }

        type Lender {
            id: ID!
            borrower: Borrower! @join(on:account)
        }

        type Auditor {
            id: ID!
            account: Address!
            hash: Bytes32! @indexed
            borrower: Borrower! @join(on:account)
        }
    "#;

        let mut expected = HashMap::new();
        expected.insert(
            "lender".to_string(),
            HashMap::from([(
                "borrower".to_string(),
                ("borrower".to_string(), "account".to_string()),
            )]),
        );
        expected.insert(
            "auditor".to_string(),
            HashMap::from([(
                "borrower".to_string(),
                ("borrower".to_string(), "account".to_string()),
            )]),
        );

        let explicit_fks =
            get_foreign_keys("foo", "bar", false, explicit_schema).unwrap();
        assert_eq!(expected, explicit_fks);
    }
}
