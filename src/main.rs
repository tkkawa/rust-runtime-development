use anyhow::Result;
use clap::Parser;

use std::fs;
use std::path::PathBuf;

use rust_runtime::create;
use rust_runtime::logger::init_logger;
use rust_runtime::start;

#[derive(Debug, Parser)]
struct Opts {
    #[clap(short, long, default_value = "/run/youki")]
    root: PathBuf,
    #[clap(short, long)]
    log: Option<PathBuf>,
    #[clap(long)]
    log_format: Option<String>,
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Debug, Parser)]
enum Commands {
    #[command(about = "")]
    Create(create::Create),
    #[command(about = "")]
    Start(start::Start),
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    init_logger();
    let root_path = PathBuf::from(&opts.root);
    fs::create_dir_all(&root_path)?;

    match opts.cmd {
        Commands::Create(create) => create.exec(root_path),
        Commands::Start(start) => start.exec(root_path)
    }
}
