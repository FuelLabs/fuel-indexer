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
        // Each level of indentation is two spaces.
        let indent = "  |".repeat(depth);

        if path.is_dir() {
            info!("{}{}/", indent.bright_cyan(), path.display().bright_cyan());
        } else {
            info!("{}{}", indent.bright_cyan(), path.display().bright_cyan());
        }
    }
    Ok(())
}
