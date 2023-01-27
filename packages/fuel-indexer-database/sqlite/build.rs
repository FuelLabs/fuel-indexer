fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        std::env::set_var("SQLX_OFFLINE", "1");
    }
}
