extern crate cartorio;
extern crate tempfile;

use cartorio::digest;
use std::io::{Read, Seek, SeekFrom, Write};
use tempfile::tempfile;
use tempfile::tempdir;

#[test]
fn test_compute_for_file() {
    let mut file = tempfile().unwrap();
    let n = file.write(b"hello world").unwrap();
    assert_eq!(n, 11);

    assert!(file.flush().is_ok());
    assert!(file.seek(SeekFrom::Start(0)).is_ok());

    assert_eq!(
        digest::compute(&file).unwrap(),
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
    );
}

#[test]
fn test_store_and_retrieve () {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("temp");

    std::fs::File::create(&file_path).expect("create file");

    assert!(digest::store(&file_path, "something").is_ok());

    let digest_opt = digest::retrieve(&file_path).unwrap();
    assert_eq!(digest_opt.unwrap(), "something");
}
