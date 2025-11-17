mod cli;
mod error;
mod file_hash;
mod jobs;
mod tui;
mod utils;

use crate::cli::Cli;
use crate::jobs::{run_workers, FileRecord};
use crate::tui::run_tui;

use clap::Parser;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse(); //.

    if cli.threads == 0 {
        eprintln!("threads must be >= 1");
        std::process::exit(2);
    }

    // Collect files
    let mut files = Vec::new();
    for path in &cli.inputs {
        let mut paths = jobs::collect_paths(path, cli.recursive)?;
        files.append(&mut paths);
    }

    if files.is_empty() {
        println!("No files found.");
        return Ok(());
    }

    // Initialize file records
    let mut records = files.into_iter().map(FileRecord::new).collect::<Vec<_>>();

    // Start worker threads
    let (status_rx, workers) = run_workers(records.clone(), cli.threads);

    // Run TUI loop
    run_tui(&mut records, status_rx)?;

    // Wait for workers
    for worker in workers {
        let _ = worker.join();
    }

    Ok(())
}