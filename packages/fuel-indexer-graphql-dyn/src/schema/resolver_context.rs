use super::node::*;
use super::paging::*;
use super::resolver::*;
use super::self_prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum DynResolverContextError {
    #[error("Failed to parse argument {0} as NodeId: {1}")]
    NodeIdParse(String, String),
    #[error("Missing argument: {0}")]
    MissingArgument(String),
    #[error(transparent)]
    Paging(#[from] DynPagingError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type DynResolverContextResult<T> = anyhow::Result<T, DynResolverContextError>;

type Error = DynResolverContextError;
type Result<T> = DynResolverContextResult<T>;

#[extension_trait]
pub impl<'a> DynResolverContext<'a> for ResolverContext<'a> {
    fn resolver(&self) -> &DynResolver {
        self.data::<DynResolver>().unwrap()
    }
    fn parent<T: 'static>(&self) -> Result<&T> {
        self.parent_value.try_downcast_ref::<T>().map_err(|_| {
            Error::Other(anyhow::anyhow!(
                "failed to downcast parent value to {}",
                std::any::type_name::<T>()
            ))
        })
    }
    fn parent_node(&self) -> Result<&DynNode> {
        self.parent::<DynNode>()
    }
    fn field_name(&self) -> String {
        self.path_node.unwrap().field_name().to_string()
    }

    // Argument getter methods
    fn get_arg(&self, name: &str) -> Option<ValueAccessor> {
        self.args.get(name)
    }
    fn get_arg_u64(&self, name: &str) -> Result<Option<u64>> {
        self.args
            .get(name)
            .map(|arg| {
                arg.u64().map_err(|_e| {
                    Error::Other(anyhow::anyhow!(
                        "Failed to get argument {} as u64",
                        name
                    ))
                })
            })
            .transpose()
    }
    fn get_arg_string(&self, name: &str) -> Result<Option<&str>> {
        self.args
            .get(name)
            .map(|arg| {
                arg.string().map_err(|_e| {
                    Error::Other(anyhow::anyhow!(
                        "Failed to get argument {} as string",
                        name
                    ))
                })
            })
            .transpose()
    }
    // fn get_arg_node_id(&self, name: &str) -> Result<Option<data::ObjId>> {
    //     self.args
    //         .get(name)
    //         .map(|arg| {
    //             arg.node_id().map_err(|_e| {
    //                 Error::Other(anyhow::anyhow!(
    //                     "Failed to get argument {} as NodeId",
    //                     name
    //                 ))
    //             })
    //         })
    //         .transpose()
    // }
    fn get_arg_object(&self, name: &str) -> Result<Option<ObjectAccessor>> {
        self.args
            .get(name)
            .map(|arg| {
                arg.object().map_err(|_e| {
                    Error::Other(anyhow::anyhow!(
                        "Failed to get argument {} as object",
                        name
                    ))
                })
            })
            .transpose()
    }
    fn get_arg_nn(&self, name: &str) -> Result<ValueAccessor> {
        self.get_arg(name)
            .ok_or_else(|| Error::MissingArgument(name.into()))
    }
    fn get_arg_string_nn(&self, name: &str) -> Result<&str> {
        self.get_arg_string(name)?
            .ok_or_else(|| Error::MissingArgument(name.into()))
    }
    // fn get_arg_node_id_nn(&self, name: &str) -> Result<data::ObjId> {
    //     self.get_arg_node_id(name)?
    //         .ok_or_else(|| Error::MissingArgument(name.into()))
    // }
    fn get_arg_list_nn_string_nn(&self, name: &str) -> Result<Vec<String>> {
        self.get_arg_nn(name)?
            .list()
            .map_err(|_e| {
                Error::Other(anyhow::anyhow!("Failed to get argument {} as List", name))
            })?
            .iter()
            .map(|id| {
                id.string()
                    .map_err(|_e| {
                        Error::Other(anyhow::anyhow!(
                            "Failed to get argument {} as String",
                            name
                        ))
                    })
                    .map(ToString::to_string)
            })
            .collect()
    }
    fn get_arg_list_nn_node_id_nn(&self, _name: &str) -> Result<Vec<DynNodeId>> {
        unimplemented!()
        // self.get_arg_list_nn_string_nn(name)?
        //     .iter()
        //     .map(|id| {
        //         id.parse()
        //             .map_err(|_e| Error::NodeIdParse(name.to_string(), id.clone()))
        //     })
        //     .collect()
    }
    fn get_paging(&self) -> Result<DynPaging> {
        let first = self.get_arg_u64("first")?;
        let after = self.get_arg_string("after")?.map(ToString::to_string);
        let last = self.get_arg_u64("last")?;
        let before = self.get_arg_string("before")?.map(ToString::to_string);
        let args = DynPagingArgs {
            first,
            after,
            last,
            before,
        };
        Ok(DynPaging::try_from(args)?)
    }
}
