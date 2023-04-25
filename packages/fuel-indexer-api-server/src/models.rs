use crate::{ApiError, ApiResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::types::JsonValue;

/// Alias for an arbitrary `sqlx` query response.
pub type QueryResponse = Value;

/// Verify signature request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VerifySignatureRequest {
    /// Signature to verify.
    pub signature: String,

    /// Message used to recover public key from signature.
    pub message: String,
}

/// Type of page results.
pub enum PageType {
    /// Paginated page results.
    Paginated(Paginated),

    /// Plain page results.
    Plain(QueryResponse),
}

impl TryFrom<QueryResponse> for PageType {
    type Error = ApiError;
    fn try_from(val: QueryResponse) -> ApiResult<Self> {
        let val = val.as_array().expect("Bad page_info.").to_owned();
        let mut obj = val[0].as_object().expect("Bad page_info.").to_owned();

        if let Some(info) = obj.remove("page_info") {
            let PageInfo {
                has_next_page,
                limit,
                offset,
                pages,
                total_count,
            } = info.into();

            return Ok(Self::Paginated(Paginated {
                has_next_page,
                limit,
                offset,
                pages,
                total_count,
                data: QueryResponse::from(val),
            }));
        }

        Ok(Self::Plain(QueryResponse::from(val)))
    }
}

/// Pagination metadata returned from `sqlx` queries.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PageInfo {
    /// If the pagination result inclues another page that can be fetched.
    has_next_page: bool,

    /// Amount of results to include in a single page.
    limit: usize,

    /// Offset at which to read next `limit` amount of items.
    offset: usize,

    /// Total amount of pages included in this query result.
    pages: usize,

    /// Total amount of items included in this query result.
    total_count: usize,
}

impl From<Value> for PageInfo {
    fn from(val: Value) -> Self {
        Self {
            has_next_page: val["has_next_page"].as_bool().unwrap(),
            limit: val["limit"].as_u64().unwrap() as usize,
            offset: val["offset"].as_u64().unwrap() as usize,
            pages: val["pages"].as_u64().unwrap() as usize,
            total_count: val["total_count"].as_u64().unwrap() as usize,
        }
    }
}

/// Paginated `PageType`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Paginated {
    /// If the pagination result inclues another page that can be fetched.
    has_next_page: bool,

    /// Amount of results to include in a single page.
    limit: usize,

    /// Offset at which to read next `limit` amount of items.
    offset: usize,

    /// Total amount of pages included in this query result.
    pages: usize,

    /// Total amount of items included in this query result.
    total_count: usize,

    /// List of query results.
    data: QueryResponse,
}

impl From<Paginated> for JsonValue {
    fn from(val: Paginated) -> Self {
        let Paginated {
            has_next_page,
            limit,
            offset,
            pages,
            total_count,
            data,
        } = val;
        json!({
            "has_next_page": has_next_page,
            "limit": limit,
            "offset": offset,
            "pages": pages,
            "total_count": total_count,
            "data": data
        })
    }
}

impl From<Paginated> for axum::Json<JsonValue> {
    fn from(val: Paginated) -> Self {
        let Paginated {
            has_next_page,
            limit,
            offset,
            pages,
            total_count,
            data,
        } = val;
        axum::Json(json!({
            "has_next_page": has_next_page,
            "limit": limit,
            "offset": offset,
            "pages": pages,
            "total_count": total_count,
            "data": data
        }))
    }
}

/// GraphQL request query parameters.
#[derive(Deserialize)]
pub(crate) struct GraphQLQuery {
    pub(crate) page_info: Option<bool>,
}
