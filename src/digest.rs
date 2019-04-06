extern crate hex;
extern crate sha2;
extern crate xattr;

use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path};

use sha2::{Digest, Sha256};


/// The xattr field to store the digest.
///
const DIGEST_XATTR: &'static str = "digest";


/// Stores digest information into a file.
///
///
/// # Arguments
///
/// * `filepath` - path to the file where the digest should be stored into.
///
///
/// # Remarks
///
/// The filesystem where the file lives must support having extended
/// attributes set & get from files.
///
pub fn store(filepath: &Path, digest: &str) -> io::Result<()> {
    xattr::set(filepath, DIGEST_XATTR, digest.as_bytes())
}


/// Retrieves digest information from a file.
///
/// # Remarks
///
/// The filesystem where the file lives must support having extended
/// attributes set & get from files.
///
/// # Panics
///
/// The method might panic if the stored string is not utf8.
///
pub fn retrieve(filepath: &Path) -> io::Result<Option<String>> {
    let get_opt = xattr::get(filepath, DIGEST_XATTR)?;

    match get_opt {
        Some(v) => {
            let value = std::str::from_utf8(&v).unwrap();

            return Ok(Some(value.to_string()));
        },
        None => Ok(None),
    }
}


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
pub fn compute_for_file(filepath: &Path) -> io::Result<String> {
    compute(File::open(filepath)?)
}


/// Computes the digest of a file and stores it in its xattr.
///
///
pub fn compute_for_file_and_store(filepath: &Path) -> io::Result<String> {
    let digest = compute_for_file(filepath)?;

    store(filepath, &digest)?;

    Ok(digest)
}

pub fn retrieve_or_compute_and_store(filepath: &Path) -> io::Result<String> {
    let digest_opt = retrieve(filepath)?;

    if let Some(d) = digest_opt {
        return Ok(d);
    }
    
    compute_for_file_and_store(filepath)
}


/// Computes the digest from the contents of a string.
///
/// # Arguments
///
/// * `content` - the string whose digest is meant to be computed.
///
pub fn compute_for_string(content: &str) -> String {
    hex::encode(Sha256::digest(content.as_bytes()).as_slice())
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
