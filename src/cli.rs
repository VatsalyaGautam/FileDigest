use clap::Parser;
use std::path::PathBuf;
use crate::error::AppError;
use std::os::unix::fs::FileTypeExt;

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
use std::collections::HashSet;
use std::fs;

impl Cli {
    pub fn validate(&self) -> Result<(), AppError> {

        if self.threads == 0 {
            return Err(AppError::InvalidArgument(
                "`--threads` must be greater than 0".into(),
            ));
        }

     
        let mut seen = HashSet::new();
        for path in &self.inputs {
            let as_string = path.to_string_lossy().to_string();
            if !seen.insert(as_string.clone()) {
                return Err(AppError::InvalidArgument(
                    format!("Duplicate input path: {}", as_string),
                ));
            }
        }

       
        for path in &self.inputs {
            if !path.exists() {
                return Err(AppError::NotFound(
                    format!("{:?}", path)
                ));
            }
        }

       
        if !self.recursive {
            for path in &self.inputs {
                if path.is_dir() {
                    return Err(AppError::InvalidArgument(
                        format!(
                            "Directory {:?} passed, but recursion is disabled. \
                             Enable with --recursive.",
                            path
                        ),
                    ));
                }
            }
        }

     
        for path in &self.inputs {
            let meta = fs::metadata(path)?;
            let ftype = meta.file_type();

            if ftype.is_symlink() {
                return Err(AppError::InvalidArgument(
                    format!("Unsupported symlink: {:?}", path),
                ));
            }

            if ftype.is_socket() {
                return Err(AppError::InvalidArgument(
                    format!("Unsupported socket file: {:?}", path),
                ));
            }

            if ftype.is_fifo() {
                return Err(AppError::InvalidArgument(
                    format!("Unsupported pipe/fifo: {:?}", path),
                ));
            }

            if ftype.is_char_device() || ftype.is_char_device() {
                return Err(AppError::InvalidArgument(
                    format!("Unsupported device file: {:?}", path),
                ));
            }
        }

        Ok(())
    }
}

