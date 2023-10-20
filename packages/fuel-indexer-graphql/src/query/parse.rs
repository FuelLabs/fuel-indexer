use std::collections::HashMap;

use async_graphql_parser::{
    types::{
        BaseType, Directive, FragmentDefinition, OperationDefinition, OperationType,
        Selection, VariableDefinition,
    },
    Positioned,
};
use async_graphql_value::{Name, Value, Variables};
use fuel_indexer_lib::graphql::{parser::InternalType, ParsedGraphQLSchema};

use crate::{
    query::arguments::{parse_argument_into_param, FilterType, ParamType},
    query::{ParsedSelection, QueryKind},
    GraphqlError, GraphqlResult,
};

/// Contains information about a successfully-parsed user operation.
#[derive(Debug)]
pub struct ParsedOperation {
    pub name: Option<String>,
    pub selections: Vec<ParsedSelection>,
    pub ty: OperationType,
    pub directives: Vec<Positioned<Directive>>,
}
impl ParsedOperation {
    /// Creates a `ParsedOperation` from a user's operation.
    pub fn generate(
        schema: &ParsedGraphQLSchema,
        operation_def: &OperationDefinition,
        fragments: &HashMap<Name, Positioned<FragmentDefinition>>,
        name: Option<String>,
        variables: &Variables,
    ) -> GraphqlResult<Self> {
        let variable_definitions: HashMap<String, Positioned<VariableDefinition>> =
            operation_def.variable_definitions.iter().fold(
                HashMap::new(),
                |mut map, positioned_var_def| {
                    let var_name = positioned_var_def.node.name.to_string();
                    map.insert(var_name, positioned_var_def.clone());
                    map
                },
            );
        match operation_def.ty {
            OperationType::Query => Ok(Self {
                name,
                ty: operation_def.ty,
                directives: operation_def.directives.clone(),
                selections: parse_selections(
                    &operation_def.selection_set.node.items,
                    fragments,
                    schema,
                    None,
                    variables,
                    &variable_definitions,
                )?,
            }),
            OperationType::Mutation => {
                Err(GraphqlError::OperationNotSupported("Mutation".to_string()))
            }
            OperationType::Subscription => Err(GraphqlError::OperationNotSupported(
                "Subscription".to_string(),
            )),
        }
    }
}

/// Parses selections from an `OperationDefinition` into a list of `ParsedSelection`s.
fn parse_selections(
    selections: &[Positioned<Selection>],
    fragments: &HashMap<Name, Positioned<FragmentDefinition>>,
    schema: &ParsedGraphQLSchema,
    parent_obj: Option<&String>,
    variables: &Variables,
    variable_definitions: &HashMap<String, Positioned<VariableDefinition>>,
) -> GraphqlResult<Vec<ParsedSelection>> {
    // We're using a fold operation here in order to collect nodes from both field selections
    // as well as selections from fragment defintions.
    let parsed_selections = selections.iter().try_fold(vec![], |mut v, selection| {
        Ok(match &selection.node {
            Selection::Field(f) => {
                // Check for @skip and @include (the only directives required by the GraphQL spec)
                let skip_include = f
                    .node
                    .directives
                    .iter()
                    .filter(|d| {
                        let name = d.node.name.node.to_string();
                        name == "skip" || name == "include"
                    })
                    .collect::<Vec<&Positioned<Directive>>>();

                for d in skip_include {
                    if let Some(val) = d.node.get_argument("if") {
                        match &val.node {
                            Value::Variable(var) => {
                                let key = var.to_string();
                                if variable_definitions.contains_key(&key) {
                                    if let Some(variable_value) = variables.get(var) {
                                        if let Value::Boolean(cond) =
                                            variable_value.clone().into_value()
                                        {
                                            if (d.node.name.node == "skip" && cond)
                                                || (d.node.name.node == "include"
                                                    && !cond)
                                            {
                                                return Ok(v);
                                            }
                                        }
                                    } else {
                                        // The only way we get a key to use in the variables map is by
                                        // iterating through the existing variables, so we shouldn't be
                                        // able to use a key that doesn't exist.
                                        unreachable!()
                                    }
                                } else {
                                    return Err(GraphqlError::UndefinedVariable(
                                        var.to_string(),
                                    ));
                                }
                            }
                            Value::Boolean(cond) => {
                                if (d.node.name.node == "skip" && *cond)
                                    || (d.node.name.node == "include" && !cond)
                                {
                                    return Ok(v);
                                }
                            }
                            _ => continue,
                        }
                    }
                }

                let field_type = schema.graphql_type(parent_obj, &f.node.name.node);

                // If this function was called with a parent object, then the ParsedSelection
                // will NOT be a root level object. Thus, it needs to be parsed into the
                // correct type of ParsedSelection.
                if let Some(parent) = parent_obj {
                    let arguments = f
                        .node
                        .arguments
                        .iter()
                        .map(|(arg, value)| {
                            if let Value::Variable(var) = &value.node {
                                if variable_definitions.contains_key(&var.to_string()) {
                                    if let Some(variable_value) = variables.get(var) {
                                        parse_argument_into_param(
                                            field_type,
                                            &arg.to_string(),
                                            variable_value.clone().into_value(),
                                            schema,
                                        )
                                    } else {
                                        // The only way we get a key to use in the variables map is by
                                        // iterating through the existing variables, so we shouldn't be
                                        // able to use a key that doesn't exist.
                                        unreachable!()
                                    }
                                } else {
                                    Err(GraphqlError::UndefinedVariable(var.to_string()))
                                }
                            } else {
                                parse_argument_into_param(
                                    field_type,
                                    &arg.to_string(),
                                    value.node.clone(),
                                    schema,
                                )
                            }
                        })
                        .collect::<GraphqlResult<Vec<ParamType>>>()?;
                    let has_no_subselections = f.node.selection_set.node.items.is_empty();

                    // List fields require a different function than the one used for objects,
                    // and internal types (e.g. pagination helper types) don't have tables in the database.
                    let (is_list_field, internal_type) = if let Some(parent) = parent_obj
                    {
                        let key = format!("{}.{}", parent, f.node.name.node);
                        if let Some((f_def, _)) = schema.field_defs().get(&key) {
                            match &f_def.ty.node.base {
                                BaseType::Named(t) => {
                                    if let Some(internal) =
                                        schema.internal_types().get(&t.to_string())
                                    {
                                        (false, Some(internal))
                                    } else {
                                        (false, None)
                                    }
                                }
                                BaseType::List(inner_type) => {
                                    if let BaseType::Named(t) = &inner_type.base {
                                        if let Some(internal) =
                                            schema.internal_types().get(&t.to_string())
                                        {
                                            (true, Some(internal))
                                        } else {
                                            (true, None)
                                        }
                                    } else {
                                        return Err(GraphqlError::ListsOfLists);
                                    }
                                }
                            }
                        } else {
                            (false, None)
                        }
                    } else {
                        (false, None)
                    };

                    let selection_node = if let Some(t) = internal_type {
                        let mut fields = parse_selections(
                            &f.node.selection_set.node.items,
                            fragments,
                            schema,
                            field_type,
                            variables,
                            variable_definitions,
                        )?;
                        let key = format!("{}.{}", parent, f.node.name.node);
                        let entity = if let Some(pagination_type) =
                            schema.field_type_mappings().get(&key)
                        {
                            if let Some(underlying_obj) =
                                schema.pagination_types().get(pagination_type)
                            {
                                underlying_obj.clone()
                            } else {
                                return Err(GraphqlError::CouldNotGetBaseEntityType(
                                    f.node.name.node.to_string(),
                                ));
                            }
                        } else {
                            return Err(GraphqlError::CouldNotGetBaseEntityType(
                                f.node.name.node.to_string(),
                            ));
                        };

                        match t {
                            InternalType::Edge => {
                                let cursor = if let Some(idx) =
                                    fields.iter().position(|f| f.name() == *"cursor")
                                {
                                    let c = fields.swap_remove(idx);
                                    Box::new(Some(c))
                                } else {
                                    Box::new(None)
                                };

                                let node = if let Some(idx) =
                                    fields.iter().position(|f| f.name() == *"node")
                                {
                                    let n = fields.swap_remove(idx);
                                    Box::new(Some(n))
                                } else {
                                    Box::new(None)
                                };

                                ParsedSelection::Edge {
                                    name: f.node.name.node.clone(),
                                    cursor,
                                    entity,
                                    node,
                                }
                            }
                            InternalType::PageInfo => {
                                let backing_obj = if let Some(underlying_obj) =
                                    schema.pagination_types().get(parent)
                                {
                                    underlying_obj.clone()
                                } else {
                                    return Err(GraphqlError::CouldNotGetBaseEntityType(
                                        parent.to_owned(),
                                    ));
                                };

                                ParsedSelection::PageInfo {
                                    name: f.node.name.node.clone(),
                                    alias: f.node.alias.clone(),
                                    fields,
                                    parent_entity: backing_obj,
                                }
                            }
                            _ => {
                                return Err(GraphqlError::InternalTypeParseError(entity))
                            }
                        }
                    } else if has_no_subselections {
                        ParsedSelection::Scalar {
                            name: f.node.name.node.clone(),
                            alias: f.node.alias.clone(),
                            parent_entity: parent.clone(),
                        }
                    } else {
                        let fields = parse_selections(
                            &f.node.selection_set.node.items,
                            fragments,
                            schema,
                            field_type,
                            variables,
                            variable_definitions,
                        )?;
                        let entity_type = if let Some(ty) = field_type {
                            ty.clone()
                        } else {
                            return Err(GraphqlError::CouldNotGetBaseEntityType(
                                f.node.name.node.to_string(),
                            ));
                        };

                        ParsedSelection::Object {
                            name: f.node.name.node.clone(),
                            alias: f.node.alias.clone(),
                            fields,
                            parent_entity: parent.to_string(),
                            is_part_of_list: is_list_field,
                            arguments,
                            entity_type,
                        }
                    };

                    if is_list_field {
                        let entity_type = if let Some(ty) = field_type {
                            ty.clone()
                        } else {
                            return Err(GraphqlError::CouldNotGetBaseEntityType(
                                f.node.name.node.to_string(),
                            ));
                        };

                        v.push(ParsedSelection::List {
                            node: Box::new(selection_node),
                            name: f.node.name.node.clone(),
                            alias: f.node.alias.clone(),
                            obj_type: entity_type,
                        });
                        v
                    } else {
                        v.push(selection_node);
                        v
                    }
                } else if let Some(query_type) =
                    schema.query_response_type(&f.node.name.node)
                {
                    let fields = parse_selections(
                        &f.node.selection_set.node.items,
                        fragments,
                        schema,
                        Some(&query_type),
                        variables,
                        variable_definitions,
                    )?;
                    let (kind, query_type) = if query_type.contains("Connection") {
                        (QueryKind::Connection, query_type.replace("Connection", ""))
                    } else {
                        (QueryKind::Object, query_type)
                    };
                    let mut arguments = f
                        .node
                        .arguments
                        .iter()
                        .map(|(arg, value)| {
                            if let Value::Variable(var) = &value.node {
                                if variable_definitions.contains_key(&var.to_string()) {
                                    if let Some(variable_value) = variables.get(var) {
                                        parse_argument_into_param(
                                            Some(&query_type),
                                            &arg.to_string(),
                                            variable_value.clone().into_value(),
                                            schema,
                                        )
                                    } else {
                                        // The only way we get a key to use in the variables map is by
                                        // iterating through the existing variables, so we shouldn't be
                                        // able to use a key that doesn't exist.
                                        unreachable!()
                                    }
                                } else {
                                    Err(GraphqlError::UndefinedVariable(var.to_string()))
                                }
                            } else {
                                parse_argument_into_param(
                                    Some(&query_type),
                                    &arg.to_string(),
                                    value.node.clone(),
                                    schema,
                                )
                            }
                        })
                        .collect::<GraphqlResult<Vec<ParamType>>>()?;

                    if let QueryKind::Object = kind {
                        if !arguments.iter().any(|a| {
                            matches!(
                                a,
                                ParamType::Filter(FilterType::RecordSelection(_, _))
                            )
                        }) {
                            return Err(GraphqlError::ObjectQueryNeedsIdArg);
                        } else {
                            arguments.push(ParamType::Limit(1));
                        }
                    }

                    v.push(ParsedSelection::QueryRoot {
                        name: f.node.name.node.clone(),
                        alias: f.node.alias.clone(),
                        fields,
                        arguments,
                        root_entity_type: query_type,
                        kind,
                    });
                    return Ok(v);
                } else {
                    return Err(GraphqlError::RootNeedsToBeAQuery);
                }
            }
            Selection::FragmentSpread(frag_spread) => {
                if let Some(definition) =
                    fragments.get(&frag_spread.node.fragment_name.node)
                {
                    let selections = &definition.node.selection_set.node.items;
                    let mut sub_selections = parse_selections(
                        selections,
                        fragments,
                        schema,
                        parent_obj,
                        variables,
                        variable_definitions,
                    )?;
                    v.append(&mut sub_selections);
                    v
                } else {
                    return Err(GraphqlError::FragmentResolverFailed);
                }
            }
            // TODO: Figure out what to do with this
            Selection::InlineFragment(_) => todo!(),
        })
    });

    parsed_selections
}
