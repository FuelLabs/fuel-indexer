use crate::cli::RevertCommand;


pub fn init(command: RevertCommand) -> anyhow::Result<()> {
    let manifest = command.manifest.unwrap_or_else(|| {
        let mut path = command.path.unwrap_or_else(|| PathBuf::from("."));
        path.push("Cargo.toml");
        path.to_str().unwrap().to_string()
    });
    let manifest = Manifest::from_path(&manifest)?;
    let indexer = Indexer::from_manifest(&manifest)?;
    let indexer = indexer.revert()?;
    let indexer = indexer.deploy()?;
    indexer.wait_for_ready()?;
    Ok(())
}
