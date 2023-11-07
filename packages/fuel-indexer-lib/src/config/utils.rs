use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref NETWORKS: HashMap<String, (String, u64)> = HashMap::from_iter(
        default_network_values()
            .iter()
            .map(|n| (n.to_string(), (format!("{n}.fuel.network"), 80)))
    );
}

/// Pass default network names to `clap`.
pub fn default_network_values() -> &'static [&'static str] {
    &["beta-3", "beta-4", "beta-5"]
}

/// Attach a protocol to a host and port.
pub fn derive_http_url(host: &String, port: &String) -> String {
    let protocol = match port.as_str() {
        "443" | "4443" => "https",
        _ => "http",
    };

    format!("{protocol}://{host}:{port}")
}
