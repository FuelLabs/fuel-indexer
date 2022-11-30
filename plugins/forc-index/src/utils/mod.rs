pub mod defaults;

pub(crate) fn dasherize_to_underscore(s: &str) -> String {
    str::replace(s, "-", "_")
}
