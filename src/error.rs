use thiserror::Error;
use std::io;
#[derive(Error, Debug)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("file not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}