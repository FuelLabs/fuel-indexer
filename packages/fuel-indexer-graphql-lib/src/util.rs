//! Utilities for `async_graphql::dynamic`

use super::prelude::*;
use async_graphql::dynamic::*;

#[extension_trait]
pub impl<'a> ResolverContextUtilExt<'a> for ResolverContext<'a> {
    fn parent_value<T>(&self) -> &'a T
    where
        T: 'static,
    {
        self.parent_value
            .try_downcast_ref::<T>()
            .unwrap_or_else(|_| {
                panic!(
                    "Parent value casting failed. Expected: {}",
                    std::any::type_name::<T>()
                )
            })
    }
}
