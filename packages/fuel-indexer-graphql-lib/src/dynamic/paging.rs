use super::prelude::*;

pub type DynamicCursor = Cursor;

#[derive(Clone, PartialEq, Eq)]
pub struct DynamicPageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<DynamicCursor>,
    pub end_cursor: Option<DynamicCursor>,
}
