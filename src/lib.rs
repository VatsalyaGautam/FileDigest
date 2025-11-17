pub mod cli;
pub mod error;
pub mod file_hash;
pub mod jobs;
pub mod utils;

// Re-export commonly used items for easier access in tests
pub use cli::Cli;
pub use file_hash::hash_file;
pub use jobs::{collect_paths, run_workers, FileRecord, JobStatus};
pub use utils::truncate_middle;