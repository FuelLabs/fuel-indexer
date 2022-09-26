mod database;
mod executor;
mod graphql;
mod service;

pub mod assets {
    pub const MANIFEST: &str = include_str!("./../assets/simple_wasm.yaml");
    pub const BAD_MANIFEST: &str = include_str!("./../assets/bad_simple_wasm.yaml");
    pub const BAD_WASM_BYTES: &[u8] = include_bytes!("./../assets/bad_simple_wasm.wasm");
    pub const WASM_BYTES: &[u8] = include_bytes!("./../assets/simple_wasm.wasm");
}
