//! Module for GraphQL spec types.
//!
//! Graph Theory: https://en.wikipedia.org/wiki/Graph_theory
//! Directed Graph: https://en.wikipedia.org/wiki/Directed_graph
//! Glossary: https://en.wikipedia.org/wiki/Glossary_of_graph_theory
//! GraphQL Spec: https://spec.graphql.org/draft/
//! GraphQL Docs: https://graphql.org/learn/
//! GraphQL Cursor Connections Spec: https://relay.dev/graphql/connections.htm
//! TAO Article: https://engineering.fb.com/2013/06/25/core-data/tao-the-power-of-the-graph/
//! TAO Paper: https://research.facebook.com/publications/tao-facebooks-distributed-data-store-for-the-social-graph/

pub mod connection;
pub mod node;
pub mod paging;
pub mod query;

pub(self) mod self_prelude {
    pub use super::super::self_prelude::*;
    pub use async_graphql::dynamic::*;
    pub use std::{hash::Hash, str::FromStr};
}

pub use connection::*;
pub use node::*;
pub use paging::*;
pub use query::*;
