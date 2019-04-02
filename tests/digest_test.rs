extern crate cartorio;
extern crate tempfile;

use tempfile::tempfile;
use cartorio::digest;
use std::io::{Read, Write};

#[test]
fn test_compute_for_file() {
    let mut file = tempfile().unwrap();
    writeln!(&mut file, "hello world").expect("failed to write to temp file");
}

