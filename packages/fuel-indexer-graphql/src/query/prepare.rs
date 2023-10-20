use std::collections::{HashMap, VecDeque};

use async_graphql_parser::types::OperationType;
use fuel_indexer_database_types::DbType;
use fuel_indexer_lib::graphql::ParsedGraphQLSchema;
use indexmap::{IndexMap, IndexSet};
use petgraph::graph::{Graph, NodeIndex};

use crate::{
    query::arguments::QueryParams,
    query::{ParsedSelection, QueryKind},
    GraphqlError, GraphqlResult,
};

use super::parse::ParsedOperation;

/// A `CommonTable` holds all of the necessary information to create common table
/// expressions (CTEs), which are used to efficiently query for nested objects in
/// connection types.
#[derive(Debug, Clone)]
pub struct CommonTable {
    pub name: String,
    pub table_root: PreparedSelection,
    pub root_entity_name: String,
    pub dependency_graph: DependencyGraph,
    pub fully_qualified_namespace: String,
    pub group_by_fields: Vec<String>,
    pub connecting_reference_column: Option<String>,
    pub query_params: QueryParams,
    pub db_type: DbType,
}

impl std::fmt::Display for CommonTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let PreparedSelection::Root(root) = &self.table_root {
            let mut fragments = vec![self.name.clone(), "AS (SELECT".to_string()];
            let selection_str = root
                .fields
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join(", ");

            fragments.push(selection_str);
            fragments.push(format!(
                "\nFROM {}.{}\n",
                self.fully_qualified_namespace,
                self.root_entity_name.to_lowercase()
            ));
            fragments.push(
                self.dependency_graph
                    .get_sorted_joins()
                    .unwrap()
                    .to_string(),
            );
            fragments.push(self.query_params.get_filtering_expression(&self.db_type));

            if !self.group_by_fields.is_empty() {
                fragments
                    .push(format!("\nGROUP BY {}", self.group_by_fields.join(",\n")));
            }
            fragments.append(&mut vec![
                self.query_params.get_ordering_modififer(&self.db_type),
                self.query_params.get_limit(&self.db_type),
            ]);

            fragments.push(")".to_string());

            write!(f, "{}", fragments.join(" "))
        } else {
            // TODO: This arm shouldn't be possible, but we should put guardrails here.
            write!(f, "")
        }
    }
}

/// Contains necessary information for generating joins between database tables.
#[derive(Debug, Default, Clone)]
pub struct DependencyGraph {
    pub table_node_idx_map: HashMap<String, NodeIndex>,
    pub graph: Graph<String, (String, String)>,
    pub fully_qualified_namespace: String,
}

/// Contains information about the database joins that will be needed
/// to successfully execute a user's operation.
#[derive(Debug, Default, Clone)]
pub struct Joins {
    pub join_map: IndexMap<String, IndexSet<Join>>,
    pub fully_qualified_namespace: String,
}

impl std::fmt::Display for Joins {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::type_complexity)]
        let (singular_cond_joins, multi_cond_joins): (
            Vec<(&String, &IndexSet<Join>)>,
            Vec<(&String, &IndexSet<Join>)>,
        ) = self
            .join_map
            .iter()
            .partition(|(_, join_set)| join_set.len() == 1);

        let mut joins = singular_cond_joins
            .into_iter()
            .map(|(_, j)| {
                if let Some(join) = j.first() {
                    join.to_string()
                } else {
                    "".to_string()
                }
            })
            .collect::<Vec<String>>();

        let mut combination_joins = multi_cond_joins
            .iter()
            .map(|(primary_table, join_set)| {
                let conditions = join_set
                    .iter()
                    .map(|j| {
                        format!(
                            "{}.{} = {}.{}",
                            j.referring_table,
                            j.referring_field,
                            j.primary_table,
                            j.fk_field
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(" AND ");
                format!("INNER JOIN {primary_table} ON {conditions}")
            })
            .collect::<Vec<String>>();
        joins.append(&mut combination_joins);

        write!(f, "{}", joins.join("\n"))
    }
}

/// Contains information necessary for generating database joins.
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Join {
    pub primary_table: String,
    pub referring_table: String,
    pub referring_field: String,
    pub fk_field: String,
    pub join_type: String,
}

impl std::fmt::Display for Join {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "INNER JOIN {} ON {}.{} = {}.{}",
            self.primary_table,
            self.referring_table,
            self.referring_field,
            self.primary_table,
            self.fk_field
        )
    }
}

impl DependencyGraph {
    /// Add a new node to dependency graph.
    fn add_node(&mut self, table: String) -> NodeIndex {
        if let Some(existing_node_idx) = self.table_node_idx_map.get(&table) {
            *existing_node_idx
        } else {
            let new_node_idx = self.graph.add_node(table.clone());
            self.table_node_idx_map.insert(table, new_node_idx);
            new_node_idx
        }
    }

    /// Add an edge between two existing nodes.
    fn add_edge(
        &mut self,
        parent: NodeIndex,
        child: NodeIndex,
        referring_field: String,
        foreign_key_field: String,
    ) {
        self.graph
            .add_edge(parent, child, (referring_field, foreign_key_field));
    }

    /// Returns database joins in topologically sorted order.
    fn get_sorted_joins(&self) -> GraphqlResult<Joins> {
        let toposorted_nodes =
            if let Ok(sorted) = petgraph::algo::toposort(&self.graph, None) {
                sorted
            } else {
                return Err(GraphqlError::NoCyclesAllowedInQuery);
            };

        if toposorted_nodes.is_empty() {
            return Ok(Joins::default());
        }

        let mut joins = Joins {
            fully_qualified_namespace: self.fully_qualified_namespace.clone(),
            ..Default::default()
        };

        let mut seen = vec![false; self.graph.node_count()];

        let mut stack = VecDeque::from(toposorted_nodes);

        while let Some(node_idx) = stack.pop_front() {
            if seen[node_idx.index()] {
                continue;
            }

            let mut neighbors = self
                .graph
                .neighbors_directed(node_idx, petgraph::Direction::Outgoing)
                .detach();

            while let Some(e) = neighbors.next_edge(&self.graph) {
                if let (
                    Some((referring_node, primary_node)),
                    Some((referring_field, fk_field)),
                ) = (self.graph.edge_endpoints(e), self.graph.edge_weight(e))
                {
                    if let (Some(referring_table), Some(primary_table)) = (
                        self.graph.node_weight(referring_node),
                        self.graph.node_weight(primary_node),
                    ) {
                        let join = Join {
                            primary_table: primary_table.to_owned(),
                            referring_table: referring_table.to_owned(),
                            referring_field: referring_field.to_owned(),
                            fk_field: fk_field.to_owned(),
                            join_type: "INNER".to_string(),
                        };
                        if let Some(join_set) = joins.join_map.get_mut(primary_table) {
                            join_set.insert(join);
                        } else {
                            let mut join_set = IndexSet::new();
                            join_set.insert(join);
                            joins.join_map.insert(primary_table.to_owned(), join_set);
                        }
                    }

                    stack.push_front(primary_node);
                }
            }

            seen[node_idx.index()] = true;
        }

        Ok(joins)
    }
}

/// PreparedOperation contains all of the necessary operation information to
/// generate a correct SQL query.
#[derive(Debug, Clone)]
pub struct PreparedOperation {
    pub name: Option<String>,
    pub selection_set: PreparedSelection,
    pub ctes: Vec<CommonTable>,
    pub fully_qualified_namespace: String,
    pub root_object_name: String,
    pub joins: Joins,
    pub query_parameters: QueryParams,
    pub db_type: DbType,
    pub group_by_fields: Vec<String>,
}

impl std::fmt::Display for PreparedOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.db_type {
            DbType::Postgres => {
                let mut fragments = vec![];

                if !self.ctes.is_empty() {
                    let cte_fragment = format!(
                        "WITH {}",
                        self.ctes
                            .iter()
                            .map(|cte| cte.to_string())
                            .collect::<Vec<String>>()
                            .join(",\n")
                    );
                    fragments.append(&mut vec![
                        cte_fragment,
                        format!("SELECT {}", self.selection_set),
                        format!("FROM {}s", self.root_object_name),
                        self.joins.to_string(),
                    ]);
                } else {
                    fragments.append(&mut vec![
                        format!("SELECT {}", self.selection_set),
                        format!(
                            "FROM {}.{}",
                            self.fully_qualified_namespace, self.root_object_name
                        ),
                        self.joins.to_string(),
                    ]);
                }

                fragments.push(
                    self.query_parameters
                        .get_filtering_expression(&self.db_type),
                );

                if !self.group_by_fields.is_empty() {
                    fragments
                        .push(format!("\nGROUP BY {}", self.group_by_fields.join(",\n")));
                }

                fragments.append(&mut vec![
                    self.query_parameters.get_ordering_modififer(&self.db_type),
                    self.query_parameters.get_limit(&self.db_type),
                ]);

                write!(f, "{}", fragments.join("\n"))
            }
        }
    }
}

/// Iterates through fields of a selection to get fields that are not part of a list object.
fn get_fields_from_selection(prepared_selection: &PreparedSelection) -> Vec<String> {
    // If a query has a list field, then the resultant SQL query string
    // will use an aggrate JSON function. Any field that is not included
    // in this aggregate function needs to be included in a GROUP BY statement.
    match prepared_selection {
        PreparedSelection::Field(f) => vec![f.path.clone()],
        PreparedSelection::IdReference { path, .. } => vec![path.clone()],
        PreparedSelection::List(_) => vec![],
        PreparedSelection::Object(o) => {
            let mut v = vec![];
            for f in o.fields.iter() {
                let mut fields = get_fields_from_selection(f);
                v.append(&mut fields);
            }

            v
        }
        PreparedSelection::Root(r) => {
            let mut v = vec![];
            for f in r.fields.iter() {
                let mut fields = get_fields_from_selection(f);
                v.append(&mut fields);
            }

            v
        }
    }
}

/// Prepares a string for a `ParsedOperation` for use in a database query.
pub fn prepare_operation(
    parsed_operation: &ParsedOperation,
    schema: &ParsedGraphQLSchema,
    db_type: &DbType,
) -> GraphqlResult<PreparedOperation> {
    match parsed_operation.ty {
        OperationType::Query => match db_type {
            DbType::Postgres => {
                let (
                    selection_set,
                    dependency_graph,
                    common_tables,
                    query_parameters,
                    query_kind,
                ) = prepare_query_selections(
                    schema,
                    parsed_operation.selections.clone(),
                    db_type,
                )?;

                let root_object_name = selection_set.root_name()?;
                let group_by_fields = if let QueryKind::Object = query_kind {
                    get_fields_from_selection(&selection_set)
                } else {
                    vec![]
                };

                Ok(PreparedOperation {
                    name: parsed_operation.name.clone(),
                    selection_set,
                    ctes: common_tables,
                    fully_qualified_namespace: schema.fully_qualified_namespace(),
                    root_object_name,
                    joins: dependency_graph.get_sorted_joins()?,
                    query_parameters,
                    db_type: db_type.to_owned(),
                    group_by_fields,
                })
            }
        },
        OperationType::Mutation => {
            Err(GraphqlError::OperationNotSupported("Mutation".to_string()))
        }
        OperationType::Subscription => Err(GraphqlError::OperationNotSupported(
            "Subscription".to_string(),
        )),
    }
}
/// Scalar field in `PreparedSelection`.
#[derive(Debug, Clone)]
pub struct Field {
    name: String,
    path: String,
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.name, self.path)
    }
}

/// Object field in `PreparedSelection`.
#[derive(Debug, Clone)]
pub struct Object {
    name: Option<String>,
    fields: Vec<PreparedSelection>,
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let frag = if let Some(name) = self.name.clone() {
            format!("{name}, ")
        } else {
            "".to_string()
        };
        let fields = self
            .fields
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        write!(f, "{frag}json_build_object({fields})")
    }
}

/// Root object field in `PreparedSelection`.
#[derive(Debug, Clone)]
pub struct Root {
    name: String,
    root_entity: String,
    fields: Vec<PreparedSelection>,
    kind: QueryKind,
}

impl std::fmt::Display for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            QueryKind::Object | QueryKind::Cte => {
                let fields = self
                    .fields
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");

                write!(f, "json_build_object({fields})")
            }
            QueryKind::Connection => {
                let fragments = self
                    .fields
                    .iter()
                    .map(|selection| match selection {
                        PreparedSelection::Field(f) => {
                            format!("{}, json_agg({})", f.name, f.path)
                        }
                        _ => unreachable!(),
                    })
                    .collect::<Vec<String>>();
                write!(
                    f,
                    "json_build_object('{}', json_build_object({}))",
                    self.name,
                    fragments.join(", ")
                )
            }
        }
    }
}

/// List field in `PreparedSelection`.
#[derive(Debug, Clone)]
pub struct List {
    name: String,
    selection: Box<PreparedSelection>,
    kind: QueryKind,
}

impl std::fmt::Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            QueryKind::Object => {
                write!(f, "{}, json_agg({})", self.name, self.selection)
            }
            QueryKind::Connection => {
                write!(f, "{} AS {}", self.selection, self.name.replace('\'', ""))
            }
            QueryKind::Cte => {
                write!(
                    f,
                    "json_agg({}) AS {}",
                    self.selection,
                    self.name.replace('\'', "")
                )
            }
        }
    }
}

/// Representation of fields and objects to be selected as part of a user's operation.
#[derive(Debug, Clone)]
pub enum PreparedSelection {
    Field(Field),
    List(List),
    Object(Object),
    Root(Root),
    IdReference { name: String, path: String },
}

impl std::fmt::Display for PreparedSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            PreparedSelection::Object(o) => write!(f, "{o}"),
            PreparedSelection::Field(field) => write!(f, "{field}"),
            PreparedSelection::List(l) => write!(f, "{l}"),
            PreparedSelection::Root(r) => write!(f, "{r}"),
            PreparedSelection::IdReference { name, path } => {
                write!(f, "{path} AS {name}")
            }
        }
    }
}

impl PreparedSelection {
    /// Return name associated with selection.
    fn name(&self) -> String {
        match &self {
            PreparedSelection::Field(f) => f.name.clone(),
            PreparedSelection::List(l) => l.name.clone(),
            PreparedSelection::Object(o) => {
                if let Some(n) = &o.name {
                    n.to_owned()
                } else {
                    unreachable!()
                }
            }
            PreparedSelection::Root(r) => r.root_entity.clone(),
            PreparedSelection::IdReference { name, .. } => name.to_owned(),
        }
    }

    /// Return name of root object.
    fn root_name(&self) -> GraphqlResult<String> {
        match &self {
            PreparedSelection::Root(r) => Ok(r.root_entity.to_lowercase()),
            _ => Err(GraphqlError::RootNameOnNonRootObj),
        }
    }
}

/// Prepare selections for a GraphQL query.
fn prepare_query_selections(
    schema: &ParsedGraphQLSchema,
    parsed_selections: Vec<ParsedSelection>,
    db_type: &DbType,
) -> GraphqlResult<(
    PreparedSelection,
    DependencyGraph,
    Vec<CommonTable>,
    QueryParams,
    QueryKind,
)> {
    let mut query_parameters = QueryParams::default();
    let mut dependency_graph = DependencyGraph::default();
    let mut common_tables: Vec<CommonTable> = vec![];

    // TODO: This probably needs to be an iterator
    let (prepared_query_selections, query_kind) = match parsed_selections[0].clone() {
        ref root @ ParsedSelection::QueryRoot { ref kind, .. } => (
            prepare_selection(
                root.clone(),
                schema,
                db_type,
                &mut dependency_graph,
                &mut query_parameters,
                &mut common_tables,
                kind,
            )?,
            kind.clone(),
        ),
        _ => return Err(GraphqlError::RootNeedsToBeAQuery),
    };

    Ok((
        prepared_query_selections,
        dependency_graph,
        common_tables,
        query_parameters,
        query_kind,
    ))
}

/// Parses a `ParsedSelection` into a collection of strings that will
/// be used for generating a database query.
pub fn prepare_selection(
    parsed_selection: ParsedSelection,
    schema: &ParsedGraphQLSchema,
    db_type: &DbType,
    dependency_graph: &mut DependencyGraph,
    query_parameters: &mut QueryParams,
    common_tables: &mut Vec<CommonTable>,
    query_kind: &QueryKind,
) -> GraphqlResult<PreparedSelection> {
    match db_type {
        DbType::Postgres => {
            let fqn = schema.fully_qualified_namespace();
            match parsed_selection {
                ParsedSelection::Scalar {
                    name,
                    parent_entity,
                    alias,
                } => {
                    let field_name = alias
                        .clone()
                        .map_or(format!("'{}'", name), |a| format!("'{}'", a.node));
                    let table_path =
                        format!("{fqn}.{}.{name}", parent_entity.to_lowercase());
                    let field = Field {
                        name: field_name,
                        path: table_path,
                    };

                    Ok(PreparedSelection::Field(field))
                }
                ParsedSelection::Object {
                    name,
                    alias,
                    fields,
                    is_part_of_list,
                    arguments,
                    entity_type,
                    ..
                } => {
                    let mut obj_fields: Vec<PreparedSelection> = vec![];
                    query_parameters.add_params(
                        arguments.to_owned(),
                        format!(
                            "{}.{}",
                            schema.fully_qualified_namespace(),
                            entity_type.to_lowercase()
                        ),
                    );

                    for sn in fields {
                        if let ParsedSelection::List {
                            name: list_name,
                            obj_type,
                            ..
                        } = &sn
                        {
                            add_dependencies_for_list_selection(
                                dependency_graph,
                                schema,
                                &entity_type,
                                list_name.as_ref(),
                                obj_type,
                            );
                        } else if let Some(fk_map) = schema
                            .foreign_key_mappings()
                            .get(&entity_type.to_lowercase())
                        {
                            if let Some((fk_table, fk_field)) =
                                fk_map.get(&name.to_string())
                            {
                                let referring_node = dependency_graph.add_node(format!(
                                    "{}.{}",
                                    schema.fully_qualified_namespace(),
                                    entity_type.to_lowercase()
                                ));
                                let primary_node = dependency_graph.add_node(format!(
                                    "{}.{}",
                                    schema.fully_qualified_namespace(),
                                    fk_table.clone()
                                ));
                                dependency_graph.add_edge(
                                    referring_node,
                                    primary_node,
                                    name.clone().to_string(),
                                    fk_field.clone(),
                                );
                            }
                        }

                        let prepared_selection = prepare_selection(
                            sn,
                            schema,
                            db_type,
                            dependency_graph,
                            query_parameters,
                            common_tables,
                            query_kind,
                        )?;
                        obj_fields.push(prepared_selection);
                    }

                    let object = Object {
                        name: if !is_part_of_list {
                            let field_name =
                                alias.clone().map_or(format!("'{}'", name), |a| {
                                    format!("'{}'", a.node)
                                });
                            Some(field_name)
                        } else {
                            None
                        },
                        fields: obj_fields,
                    };

                    Ok(PreparedSelection::Object(object))
                }
                ParsedSelection::List {
                    name,
                    alias,
                    node,
                    obj_type: _,
                } => {
                    if let ParsedSelection::List { .. } = *node {
                        return Err(GraphqlError::ListsOfLists);
                    }

                    let field_name = alias
                        .clone()
                        .map_or(format!("'{}'", name), |a| format!("'{}'", a.node));

                    let list = List {
                        name: field_name,
                        selection: Box::new(prepare_selection(
                            *node,
                            schema,
                            db_type,
                            dependency_graph,
                            query_parameters,
                            common_tables,
                            &QueryKind::Cte,
                        )?),
                        kind: query_kind.clone(),
                    };

                    Ok(PreparedSelection::List(list))
                }
                ParsedSelection::QueryRoot {
                    name,
                    alias: _,
                    fields,
                    arguments,
                    kind,
                    root_entity_type,
                } => match kind {
                    QueryKind::Object => {
                        let mut obj_fields: Vec<PreparedSelection> = vec![];
                        query_parameters.add_params(
                            arguments.to_owned(),
                            format!("{}.{}", schema.fully_qualified_namespace(), name),
                        );

                        for selection_node in fields {
                            if let ParsedSelection::List {
                                name: list_name,
                                obj_type,
                                ..
                            } = &selection_node
                            {
                                add_dependencies_for_list_selection(
                                    dependency_graph,
                                    schema,
                                    &root_entity_type,
                                    list_name.as_ref(),
                                    obj_type,
                                );
                            } else if let ParsedSelection::Object { name, .. } = &selection_node {
                                if let Some(fk_map) = schema
                                    .foreign_key_mappings()
                                    .get(&root_entity_type.to_lowercase())
                                {
                                    if let Some((fk_table, fk_field)) =
                                        fk_map.get(&name.to_string())
                                    {
                                        let referring_node = dependency_graph.add_node(format!(
                                            "{}.{}",
                                            schema.fully_qualified_namespace(),
                                            root_entity_type.to_lowercase()
                                        ));
                                        let primary_node = dependency_graph.add_node(format!(
                                            "{}.{}",
                                            schema.fully_qualified_namespace(),
                                            fk_table.clone()
                                        ));
                                        dependency_graph.add_edge(
                                            referring_node,
                                            primary_node,
                                            name.clone().to_string(),
                                            fk_field.clone(),
                                        );
                                    }
                                }
                            }

                            let prepared_selection = prepare_selection(
                                selection_node,
                                schema,
                                db_type,
                                dependency_graph,
                                query_parameters,
                                common_tables,
                                &kind,
                            )?;
                            obj_fields.push(prepared_selection);
                        }

                        let object = Root {
                            name: name.to_string(),
                            fields: obj_fields,
                            root_entity: root_entity_type.to_string(),
                            kind: QueryKind::Object,
                        };

                        Ok(PreparedSelection::Root(object))
                    }
                    QueryKind::Connection => {
                        let mut cte_dep_graph = DependencyGraph {
                            fully_qualified_namespace: schema.fully_qualified_namespace(),
                            ..Default::default()
                        };

                        let mut obj_fields: Vec<PreparedSelection> = vec![];
                        for sn in fields {
                            let prepared_selection = prepare_selection(
                                sn,
                                schema,
                                db_type,
                                &mut cte_dep_graph,
                                query_parameters,
                                common_tables,
                                &QueryKind::Connection,
                            )?;
                            obj_fields.push(prepared_selection);
                        }

                        let field_keys =
                            obj_fields.iter().map(|f| f.name()).collect::<Vec<String>>();

                        for ct in common_tables.iter() {
                            if let Some(connecting_reference_column) =
                                &ct.connecting_reference_column
                            {
                                let referring_node = cte_dep_graph.add_node(format!(
                                    "{}.{}",
                                    schema.fully_qualified_namespace(),
                                    root_entity_type.to_lowercase()
                                ));
                                let primary_node =
                                    cte_dep_graph.add_node(ct.name.clone());
                                cte_dep_graph.add_edge(
                                    referring_node,
                                    primary_node,
                                    "id".to_string(),
                                    connecting_reference_column.clone(),
                                );
                            }
                        }

                        let prepared_cte_query_root = PreparedSelection::Root(Root {
                            name: name.to_string(),
                            root_entity: root_entity_type.to_string(),
                            fields: obj_fields,
                            kind: QueryKind::Cte,
                        });

                        let group_by_fields =
                            get_fields_from_selection(&prepared_cte_query_root);

                        let mut cte_query_params = QueryParams::default();
                        cte_query_params.add_params(
                            arguments.to_owned(),
                            format!(
                                "{}.{}",
                                schema.fully_qualified_namespace(),
                                root_entity_type
                            ),
                        );
                        cte_query_params.parse_pagination(db_type)?;

                        let cte = CommonTable {
                            name: name.to_string(),
                            table_root: prepared_cte_query_root.clone(),
                            root_entity_name: root_entity_type.to_string(),
                            dependency_graph: cte_dep_graph.clone(),
                            fully_qualified_namespace: schema.fully_qualified_namespace(),
                            group_by_fields,
                            connecting_reference_column: None,
                            query_params: cte_query_params,
                            db_type: db_type.clone(),
                        };

                        let selections = field_keys
                            .into_iter()
                            .map(|key| {
                                PreparedSelection::Field(Field {
                                    name: key.clone(),
                                    path: format!(
                                        "{}.{}",
                                        name.replace('\'', ""),
                                        key.replace('\'', "")
                                    ),
                                })
                            })
                            .collect::<Vec<PreparedSelection>>();

                        common_tables.push(cte);

                        let query_root = PreparedSelection::Root(Root {
                            name: name.to_string(),
                            root_entity: root_entity_type.to_string(),
                            fields: selections,
                            kind: QueryKind::Connection,
                        });

                        Ok(query_root)
                    }
                    _ => unreachable!("A query root can only have a QueryKind of either Object or Connection"),
                },
                ParsedSelection::PageInfo { .. } => unimplemented!(),
                ParsedSelection::Edge {
                    name: _,
                    cursor,
                    entity,
                    node,
                } => {
                    let mut obj_fields = vec![];
                    if cursor.is_some() {
                        let cursor_field = Field {
                            name: "'cursor'".to_string(),
                            path: format!(
                                "{}.{}.id",
                                schema.fully_qualified_namespace(),
                                entity.to_lowercase()
                            ),
                        };
                        obj_fields.push(PreparedSelection::Field(cursor_field));
                    }

                    if let Some(sn) = *node.clone() {
                        if let ParsedSelection::Object {
                            name,
                            fields,
                            entity_type,
                            ..
                        } = sn.clone()
                        {
                            let mut node_obj_fields = vec![];
                            for f in fields {

                                // If we're querying for a list field inside of a connection
                                // type query, then we need to generate a CTE.
                                if let ParsedSelection::List {
                                    name: list_name,
                                    obj_type,
                                    node: inner_obj,
                                    ..
                                } = &f
                                {
                                    let mut cte_dep_graph = DependencyGraph {
                                        fully_qualified_namespace: schema
                                            .fully_qualified_namespace(),
                                        ..Default::default()
                                    };
                                    add_dependencies_for_list_selection(
                                        &mut cte_dep_graph,
                                        schema,
                                        &entity_type,
                                        list_name.as_ref(),
                                        obj_type,
                                    );
                                    if let ParsedSelection::Object {
                                        name,
                                        parent_entity,
                                        entity_type,
                                        ..
                                    } = *inner_obj.clone()
                                    {
                                        let reference_col_name = format!(
                                            "{}_id",
                                            parent_entity.to_lowercase()
                                        );
                                        let reference_col =
                                            PreparedSelection::IdReference {
                                                name: reference_col_name.clone(),
                                                path: format!(
                                                    "{}.{}.id",
                                                    schema.fully_qualified_namespace(),
                                                    parent_entity.to_lowercase()
                                                ),
                                            };
                                        let mut inner_obj_fields =
                                            vec![reference_col.clone()];
                                        let prepared_selection = prepare_selection(
                                            f.clone(),
                                            schema,
                                            db_type,
                                            &mut cte_dep_graph,
                                            query_parameters,
                                            common_tables,
                                            query_kind,
                                        )?;
                                        inner_obj_fields.push(prepared_selection);

                                        let prepared_cte_query_root =
                                            PreparedSelection::Root(Root {
                                                name: name.to_string(),
                                                root_entity: entity_type.clone(),
                                                fields: inner_obj_fields.clone(),
                                                kind: QueryKind::Cte,
                                            });

                                        let group_by_fields = get_fields_from_selection(
                                            &prepared_cte_query_root,
                                        );

                                        let cte = CommonTable {
                                            name: name.to_string(),
                                            table_root: prepared_cte_query_root.clone(),
                                            root_entity_name: parent_entity.to_string(),
                                            dependency_graph: cte_dep_graph.clone(),
                                            fully_qualified_namespace: schema
                                                .fully_qualified_namespace(),
                                            group_by_fields,
                                            connecting_reference_column: Some(
                                                reference_col_name,
                                            ),
                                            query_params: QueryParams::default(),
                                            db_type: db_type.clone(),
                                        };
                                        common_tables.push(cte);

                                        node_obj_fields.push(PreparedSelection::Field(
                                            Field {
                                                name: format!("'{}'", name.clone()),
                                                path: format!(
                                                    "{}.{}",
                                                    name.clone(),
                                                    name
                                                ),
                                            },
                                        ));
                                    }
                                // If we're looking for a nested object at the root of the connection
                                // type query, then we need to ensure that the requisite table is
                                // properly added to the dependency graph.
                                } else if let ParsedSelection::Object { name: inner_obj_name, .. } = &f {
                                    if let Some(fk_map) = schema
                                        .foreign_key_mappings()
                                        .get(&entity_type.to_lowercase())
                                    {
                                        if let Some((fk_table, fk_field)) =
                                            fk_map.get(&inner_obj_name.to_string())
                                        {
                                            let referring_node = dependency_graph.add_node(format!(
                                                "{}.{}",
                                                schema.fully_qualified_namespace(),
                                                entity_type.to_lowercase()
                                            ));
                                            let primary_node = dependency_graph.add_node(format!(
                                                "{}.{}",
                                                schema.fully_qualified_namespace(),
                                                fk_table.clone()
                                            ));
                                            dependency_graph.add_edge(
                                                referring_node,
                                                primary_node,
                                                inner_obj_name.clone().to_string(),
                                                fk_field.clone(),
                                            );
                                        }
                                    }

                                    node_obj_fields.push(prepare_selection(
                                        f.clone(),
                                        schema,
                                        db_type,
                                        dependency_graph,
                                        query_parameters,
                                        common_tables,
                                        query_kind,
                                    )?);
                                } else {
                                    node_obj_fields.push(prepare_selection(
                                        f.clone(),
                                        schema,
                                        db_type,
                                        dependency_graph,
                                        query_parameters,
                                        common_tables,
                                        query_kind,
                                    )?);
                                }
                            }
                            obj_fields.push(PreparedSelection::Object(Object {
                                name: Some(format!("'{name}'")),
                                fields: node_obj_fields,
                            }));
                        }
                    }

                    // TODO: Alias?
                    let object = Object {
                        name: None,
                        fields: obj_fields,
                    };

                    Ok(PreparedSelection::Object(object))
                }
            }
        }
    }
}

/// Process database table dependencies needed to successfully query a list-type selection.
fn add_dependencies_for_list_selection(
    dependency_graph: &mut DependencyGraph,
    schema: &ParsedGraphQLSchema,
    parent_entity: &str,
    list_name: &str,
    list_obj_type: &str,
) {
    if let Some(fk_map) = schema
        .foreign_key_mappings()
        .get(&parent_entity.to_lowercase())
    {
        if let Some((_, fk_field)) = fk_map.get(list_name) {
            let outer_obj_node = dependency_graph.add_node(format!(
                "{}.{}",
                schema.fully_qualified_namespace(),
                parent_entity.to_lowercase()
            ));
            let inner_obj_node = dependency_graph.add_node(format!(
                "{}.{}",
                schema.fully_qualified_namespace(),
                list_obj_type.to_lowercase()
            ));
            let connecting_node = dependency_graph.add_node(format!(
                "{}.{}s_{}s",
                schema.fully_qualified_namespace(),
                parent_entity.to_lowercase(),
                list_obj_type.to_lowercase(),
            ));

            dependency_graph.add_edge(
                outer_obj_node,
                connecting_node,
                fk_field.clone(),
                format!("{}_{fk_field}", parent_entity.to_lowercase()),
            );
            dependency_graph.add_edge(
                connecting_node,
                inner_obj_node,
                format!("{}_{fk_field}", list_obj_type.to_lowercase()),
                fk_field.clone(),
            );
        }
    }
}
