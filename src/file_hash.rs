use blake3::Hasher;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::error::AppError;

/// Incremental file hashing
pub fn hash_file(path: &Path) -> Result<String, AppError> {
    let mut f = File::open(path)?;
    let mut hasher = Hasher::new();
    let mut buf = [0u8; 64 * 1024]; // 64KB buffer

    loop {
        let n = f.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }

    Ok(hex::encode(hasher.finalize().as_bytes()))
}