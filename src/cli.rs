use clap::Parser;
use std::path::PathBuf;

/// CLI definition
#[derive(Parser, Debug, Clone)]
#[command(name = "file-hasher", about = "Multithreaded file hasher with TUI")]
pub struct Cli {
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,

    #[arg(short, long, default_value_t = num_cpus::get())]
    pub threads: usize,

    #[arg(short, long, default_value_t = true)]
    pub recursive: bool,
}

// Usage in main.rs:
// let cli = Cli::parse();
