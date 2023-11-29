use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Delete {
    pub container_id: String,
    #[clap(short, long)]
    pub force: bool,
}

pub fn delete() -> Result<()> {

}