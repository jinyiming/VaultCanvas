use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub current: u64,
    pub total: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub output_path: String,
    pub bytes_processed: u64,
}

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("stego error: {0}")]
    Stego(String),
    #[error("unsupported operation: {0}")]
    Unsupported(String),
}

pub type VaultResult<T> = Result<T, VaultError>;

pub fn io_error(err: impl ToString) -> VaultError {
    VaultError::Io(err.to_string())
}

pub fn crypto_error(err: impl ToString) -> VaultError {
    VaultError::Crypto(err.to_string())
}

pub fn stego_error(err: impl ToString) -> VaultError {
    VaultError::Stego(err.to_string())
}

pub fn atomic_write(output_path: &str, bytes: &[u8]) -> VaultResult<()> {
    atomic_write_with(output_path, |file| {
        file.write_all(bytes).map_err(io_error)?;
        Ok(())
    })
}

pub fn atomic_write_with(
    output_path: &str,
    mut writer: impl FnMut(&mut fs::File) -> VaultResult<()>,
) -> VaultResult<()> {
    let target = Path::new(output_path);
    let parent = target
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    if !parent.is_dir() {
        return Err(VaultError::InvalidInput(
            "output directory does not exist".into(),
        ));
    }

    let temp_path = unique_temp_path(parent, target);
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp_path)
        .map_err(io_error)?;

    let write_result = (|| -> VaultResult<()> {
        writer(&mut file)?;
        file.sync_all().map_err(io_error)?;
        Ok(())
    })();

    drop(file);

    if let Err(err) = write_result {
        let _ = fs::remove_file(&temp_path);
        return Err(err);
    }

    fs::rename(&temp_path, target).map_err(|err| {
        let _ = fs::remove_file(&temp_path);
        io_error(err)
    })?;
    Ok(())
}

fn unique_temp_path(parent: &Path, target: &Path) -> PathBuf {
    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    let stem = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("output");
    let pid = std::process::id() as u64;
    for _ in 0..64 {
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(".{stem}.{pid:08x}.{counter:016x}.tmp"));
        if !candidate.exists() {
            return candidate;
        }
    }
    parent.join(format!(".{stem}.fallback.tmp"))
}
