extern crate hex;
extern crate sha2;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;

use sha2::{Digest, Sha256};

/// Computes the digest of a file.
///
/// # Arguments
///
/// * `reader` - the supplier of bytes that we compute the hash against.
///
pub fn compute_for_file(filepath: &PathBuf) -> io::Result<String> {
    let mut hasher = Sha256::new();
    let mut hasher_buf = [0; 1 << 12];
    let mut file = File::open(&filepath)?;

    loop {
        let amount_read = file.read(&mut hasher_buf)?;

        if amount_read == 0 {
            break;
        }
    }

    Ok(hex::encode(hasher.result().as_slice()))
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
