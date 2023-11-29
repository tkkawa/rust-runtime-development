use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Delete {
    pub container_id: String,
    #[clap(short, long)]
    pub force: bool,
}

pub fn delete(args: Delete, root_path:PathBuf) -> Result<()> {
    tracing::debug!("start deleting {}", args.container_id);
    if !container_exists(&root_path, &args.container_id)? && args.force {
        return Ok(());
    }

    let mut container = load_container(root_path, &args.container_id)?;
}