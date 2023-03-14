use crate::cli::TreeCommand;
use owo_colors::OwoColorize;
use std::path::Path;
use tracing::info;
use walkdir::WalkDir;

pub fn init(command: TreeCommand) -> anyhow::Result<()> {
    let current_dir = Path::new(".");

    //@TODO add in verbose details by file type,
    for entry in WalkDir::new(current_dir) {
        let entry = entry?;
        let path = entry.path();
        let depth = entry.depth();

        // Indent the output based on the depth of the directory.
        // Use "│ " for vertical lines and spaces, and "├─" for branches.
        let indent = "│ ".repeat(depth.saturating_sub(1))
            + match depth {
                0 => "",
                _ => "├─",
            };
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let yaml_info = " [the manifest file contains all the configuration options needed for your index]";
        let graphql_info =
            "[the GraphQL schema and types for the data your indexer will index]";
        let lib_info = "[the index execution code, written in plain rust]";

        match path.extension().and_then(|e| e.to_str()) {
            Some("yaml") => {
                info!(
                    "{}{}{}",
                    indent.bright_white(),
                    name.bright_white(),
                    yaml_info.bright_cyan(),
                )
            }
            Some("graphql") => {
                info!(
                    "{}{}{} ",
                    indent.bright_white(),
                    name.bright_white(),
                    graphql_info.bright_cyan()
                )
            }
            _ if name == "lib.rs" => {
                info!(
                    "{}{}{}",
                    indent.bright_white(),
                    name.bright_white(),
                    lib_info.bright_cyan(),
                )
            }
            _ => {
                if path.is_dir() {
                    info!("{}{}/", indent.bright_white(), name.bright_white());
                } else {
                    info!("{}{}", indent.bright_white(), name.bright_white());
                }
            }
        }
    }
    Ok(())
}
