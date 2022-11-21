use fuel_indexer_tests::fixtures::test_web::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server().await
}
