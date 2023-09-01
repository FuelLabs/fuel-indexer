use super::self_prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub struct DynPageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<Cursor>,
    pub end_cursor: Option<Cursor>,
}

pub struct DynPagingArgs {
    pub first: Option<u64>,
    pub last: Option<u64>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
}

pub enum DynPagingDirection {
    Forward,
    Backward,
}

pub struct DynPaging {
    pub direction: DynPagingDirection,
    pub count: u64,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
}

pub type DynPagingResult<T> = anyhow::Result<T, DynPagingError>;

#[derive(thiserror::Error, Debug)]
pub enum DynPagingError {
    #[error("Cannot paginate both forward and backward at the same time")]
    BothForwardAndBackward,
    #[error("Must paginate either forward or backward")]
    NoForwardOrBackward,
}

impl TryFrom<DynPagingArgs> for DynPaging {
    type Error = DynPagingError;

    fn try_from(args: DynPagingArgs) -> DynPagingResult<Self> {
        let direction = match (args.first, args.last) {
            (Some(_), Some(_)) => Err(DynPagingError::BothForwardAndBackward)?,
            (Some(_), None) => DynPagingDirection::Forward,
            (None, Some(_)) => DynPagingDirection::Backward,
            (None, None) => Err(DynPagingError::NoForwardOrBackward)?,
        };
        let count = args
            .first
            .or(args.last)
            .ok_or(DynPagingError::NoForwardOrBackward)?;
        Ok(Self {
            direction,
            count,
            before: args.before,
            after: args.after,
        })
    }
}
