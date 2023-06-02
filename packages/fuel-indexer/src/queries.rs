use anyhow::Context;
use cynic::http::ReqwestExt;
use cynic::{GraphQlResponse, Operation, QueryBuilder};
use fuel_core_client::client::schema::ConnectionArgs;
use fuel_core_client::client::schema::PageInfo;
use fuel_core_client::client::{
    schema::{
        block::{BlockByHeightArgs, Consensus, Header},
        schema,
        tx::OpaqueTransaction,
        BlockId,
    },
    PaginatedResult, PaginationRequest,
};
use std::io;

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "./assets/schema.sdl",
    graphql_type = "Query",
    variables = "ConnectionArgs"
)]
pub struct FullBlocksQuery {
    #[arguments(after: $after, before: $before, first: $first, last: $last)]
    pub blocks: FullBlockConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "./assets/schema.sdl", graphql_type = "BlockConnection")]
pub struct FullBlockConnection {
    pub edges: Vec<FullBlockEdge>,
    pub page_info: PageInfo,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "./assets/schema.sdl", graphql_type = "BlockEdge")]
pub struct FullBlockEdge {
    pub cursor: String,
    pub node: FullBlock,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "./assets/schema.sdl",
    graphql_type = "Query",
    variables = "BlockByHeightArgs"
)]
pub struct FullBlockByHeightQuery {
    #[arguments(height: $height)]
    pub block: Option<FullBlock>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "./assets/schema.sdl", graphql_type = "Block")]
pub struct FullBlock {
    pub id: BlockId,
    pub header: Header,
    pub consensus: Consensus,
    pub transactions: Vec<OpaqueTransaction>,
}

impl FullBlock {
    /// Returns the block producer public key, if any.
    pub fn block_producer(&self) -> Option<fuel_crypto::PublicKey> {
        let message = self.header.id.clone().into_message();
        match &self.consensus {
            Consensus::Genesis(_) => Some(Default::default()),
            Consensus::PoAConsensus(poa) => {
                let signature = poa.signature.clone().into_signature();
                let producer_pub_key = signature.recover(&message);
                producer_pub_key.ok()
            }
            Consensus::Unknown => None,
        }
    }
}

impl From<FullBlockConnection> for PaginatedResult<FullBlock, String> {
    fn from(conn: FullBlockConnection) -> Self {
        PaginatedResult {
            cursor: conn.page_info.end_cursor,
            has_next_page: conn.page_info.has_next_page,
            has_previous_page: conn.page_info.has_previous_page,
            results: conn.edges.into_iter().map(|e| e.node).collect(),
        }
    }
}

pub async fn full_block_by_page(
    url_str: &str,
    request: PaginationRequest<String>,
) -> io::Result<PaginatedResult<FullBlock, String>> {
    let q = FullBlocksQuery::build(request.into());

    let blocks = query(url_str, q).await?.blocks.into();

    Ok(blocks)
}

fn url(str: &str) -> reqwest::Url {
    let mut raw_url = str.to_string();
    if !raw_url.starts_with("http") {
        raw_url = format!("http://{raw_url}");
    }

    let mut url = reqwest::Url::parse(&raw_url)
        .with_context(|| format!("Invalid fuel-core URL: {str}"))
        .expect("Should be able to parse");
    url.set_path("/graphql");
    url
}

async fn query<ResponseData, Vars>(
    url_str: &str,
    q: Operation<ResponseData, Vars>,
) -> io::Result<ResponseData>
where
    Vars: serde::Serialize,
    ResponseData: serde::de::DeserializeOwned + 'static,
{
    let client = reqwest::Client::builder().build().expect("Should connect");
    let response = client
        .post(url(url_str))
        .run_graphql(q)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    decode_response(response)
}

fn decode_response<R>(response: GraphQlResponse<R>) -> io::Result<R>
where
    R: serde::de::DeserializeOwned + 'static,
{
    match (response.data, response.errors) {
        (Some(d), _) => Ok(d),
        (_, Some(e)) => Err(from_strings_errors_to_std_error(
            e.into_iter().map(|e| e.message).collect(),
        )),
        _ => Err(io::Error::new(io::ErrorKind::Other, "Invalid response")),
    }
}

pub fn from_strings_errors_to_std_error(errors: Vec<String>) -> io::Error {
    let e = errors
        .into_iter()
        .fold(String::from("Response errors"), |mut s, e| {
            s.push_str("; ");
            s.push_str(e.as_str());
            s
        });
    io::Error::new(io::ErrorKind::Other, e)
}
