extern crate cartorio;
extern crate tempfile;

use cartorio::digest;
use std::io::{Read, Seek, SeekFrom, Write};
use tempfile::tempfile;

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
