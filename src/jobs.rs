    use crossbeam_channel::{unbounded, Receiver};
use std::path::{ PathBuf};
use std::thread;
use std::time::Instant;

use crate::file_hash::hash_file;
use crate::error::AppError;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub enum Job {
    Hash(PathBuf),
}

#[derive(Debug, Clone)]
pub enum JobStatus {
    Pending,
    Working,
    Done(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub path: PathBuf,
    pub status: JobStatus,
    pub started_at: Option<Instant>,
    pub finished_at: Option<Instant>,
}

impl FileRecord {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            status: JobStatus::Pending,
            started_at: None,
            finished_at: None,
        }
    }
}

pub fn collect_paths(path: &PathBuf, recursive: bool) -> anyhow::Result<Vec<PathBuf>> {
    if path.is_file() { return Ok(vec![path.clone()]); }
    if !path.is_dir() { return Err(AppError::NotFound(path.display().to_string()).into()); }

    let files = WalkDir::new(path)
        .max_depth(if recursive { usize::MAX } else { 1 })
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    Ok(files)
}

pub fn run_workers(records: Vec<FileRecord>, thread_count: usize)
    -> (Receiver<(PathBuf, JobStatus)>, Vec<thread::JoinHandle<()>>) 
{
    let (job_tx, job_rx) = unbounded::<Job>();
    let (status_tx, status_rx) = unbounded::<(PathBuf, JobStatus)>();

    for rec in records.iter() {
        job_tx.send(Job::Hash(rec.path.clone())).unwrap();
    }
    drop(job_tx);

    let worker_count = thread_count.min(records.len()).max(1);
    let mut handles = Vec::with_capacity(worker_count);

    for i in 0..worker_count {
        let rx = job_rx.clone();
        let stx = status_tx.clone();
        let handle = thread::Builder::new()
            .name(format!("worker-{}", i))
            .spawn(move || {
                for job in rx.iter() {
                    match job {
                        Job::Hash(path) => {
                            let _ = stx.send((path.clone(), JobStatus::Working));
                            let status = match hash_file(&path) {
                                Ok(d) => JobStatus::Done(d),
                                Err(e) => JobStatus::Error(format!("{:?}", e)),
                            };
                            let _ = stx.send((path, status));
                        }
                    }
                }
            }).unwrap();
        handles.push(handle);
    }

    (status_rx, handles)
}