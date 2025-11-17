use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;
use file_hasher::{hash_file, collect_paths, FileRecord, JobStatus};

#[test]
fn test_hash_file_contents() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let mut f = File::create(&file_path).unwrap();
    writeln!(f, "hello world").unwrap();

    let digest = hash_file(&file_path).unwrap();
    assert!(!digest.is_empty());
}

#[test]
fn test_collect_paths_single_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("file1.txt");
    File::create(&file_path).unwrap();

    let files = collect_paths(&file_path, false).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0], file_path);
}

#[test]
fn test_collect_paths_recursive() {
    let dir = tempdir().unwrap();
    let subdir = dir.path().join("sub");
    fs::create_dir(&subdir).unwrap();

    let f1 = dir.path().join("a.txt");
    let f2 = subdir.join("b.txt");
    File::create(&f1).unwrap();
    File::create(&f2).unwrap();

    let files = collect_paths(&dir.path().into(), true).unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.contains(&f1));
    assert!(files.contains(&f2));
}

#[test]
fn test_file_record_creation() {
    let path = std::path::PathBuf::from("dummy.txt");
    let record = FileRecord::new(path.clone());
    assert_eq!(record.path, path);
    assert!(matches!(record.status, JobStatus::Pending));
}