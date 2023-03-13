use crate::cli::TreeCommand;
use owo_colors::OwoColorize;
use std::path::Path;
use tracing::info;
use walkdir::WalkDir;

pub fn init(command: TreeCommand) -> anyhow::Result<()> {
    let current_dir = Path::new(".");

    //@TODO add in verbose details by file type,
    //@TODO colourize
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

        // Get the name of the file or directory without the dots and slashes.
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if path.is_dir() {
            info!("{}{}/", indent.bright_cyan(), name.bright_cyan());
        } else {
            info!("{}{}", indent.bright_cyan(), name.bright_cyan());
        }
    }
    Ok(())
}
