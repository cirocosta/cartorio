extern crate hex;
extern crate sha2;

use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;

use sha2::{Digest, Sha256};

/// Computes the digest of a file.
///
/// # Arguments
///
/// * `reader` - the supplier of bytes that we compute the hash against.
///
pub fn compute(mut reader: impl Read) -> io::Result<String> {
    let mut hasher = Sha256::new();
    let mut buf = [0; 1 << 12];

    loop {
        let n = reader.read(&mut buf)?;

        if n == 0 {
            break;
        }

        hasher.input(&buf[0..n]);
    }

    Ok(hex::encode(hasher.result().as_slice()))
}

/// Computes the digest of a file.
///
/// # Arguments
///
/// * `filepath` - the path to the file to open and compute the digest.
///
pub fn compute_for_file(filepath: &PathBuf) -> io::Result<String> {
    compute(File::open(&filepath)?)
}

pub fn compute_for_string(content: &str) -> String {
    hex::encode(Sha256::digest(content.as_bytes()).as_slice())
}

fn store() {
    unimplemented!("TBD");
}

fn retrieve() {
    unimplemented!("TBD");
}

/// Adds a `sha256:` scheme to the beginning of a supplied
/// string.
///
/// ```
/// extern crate cartorio;
/// let digest = "123abc";
///
/// assert_eq!(
///     cartorio::digest::prepend_sha_scheme(&digest),
///     "sha256:123abc",
/// );
/// ```
///
pub fn prepend_sha_scheme(digest: &str) -> String {
    let mut res = String::with_capacity(2);
    res.push_str("sha256:");
    res.push_str(digest);
    res.to_string()
}
